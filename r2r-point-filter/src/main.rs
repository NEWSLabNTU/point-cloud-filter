mod config;
mod filter;
mod gui;
mod params;
mod utils;

use crate::config::Config;
use crate::filter::Filter;
use crate::gui::Gui;
use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use futures::Stream;
use futures::StreamExt;
use gui::GuiMsg;
use kiss3d::light::Light;
use kiss3d::window::Window;
use nalgebra as na;
use params::Params;
use r2r::log_error;
use r2r::Publisher;
use r2r::{geometry_msgs::msg::TransformStamped, sensor_msgs::msg::PointCloud2, QosProfile};
use r2r_msg_ext::geometry_msgs::msg::TransformStampedNalgebraExt;
use r2r_msg_ext::sensor_msgs::msg::PointCloud2NalgebraExt;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Parser)]
/// Point cloud processor.
struct Opts {
    /// config file
    #[clap(long)]
    pub config: PathBuf,
}

struct Transforms {
    pub lidar_tf: na::Isometry3<f32>,
    pub ground_tf: na::Isometry3<f32>,
}

fn main() -> Result<()> {
    // Start the ROS node
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, env!("CARGO_BIN_NAME"), "/")?;

    // Parse ROS params and load the config file.
    let params = Params::from_r2r_node(&node)?;
    let config: Config = json5::from_str(&fs::read_to_string(params.config)?)?;
    let reliable = config.reliable;

    // Start subscribers and publishers
    let qos = if reliable {
        QosProfile::default()
    } else {
        QosProfile::sensor_data()
    };

    let pcd_sub = node.subscribe::<PointCloud2>("~/input/point_cloud", qos.clone())?;
    let tf_sub = node.subscribe::<TransformStamped>("/tf", QosProfile::default())?;
    let pcd_pub = node.create_publisher::<PointCloud2>("~/output/point_cloud", qos)?;

    // Create the tokio runtime
    let rt = Runtime::new()?;

    // Wait for transform messages
    let transforms = rt
        .block_on(wait_for_tf(tf_sub))
        .ok_or_else(|| anyhow!("Unable to subscribe to /tf"))?;

    let Transforms {
        lidar_tf,
        ground_tf,
    } = transforms;

    // Create a channel from the subscriber to the filter
    let (filter_tx, filter_rx) = flume::bounded(2);

    // Create a channel from the filter to the GUI window
    let (gui_tx, gui_rx) = if params.enable_gui {
        let (gui_tx, gui_rx) = flume::bounded(2);
        (Some(gui_tx), Some(gui_rx))
    } else {
        (None, None)
    };

    // Start a forwarder worker and a filter worker and join them into one handle.
    let join = {
        let filter = Filter::new(&config, lidar_tf, ground_tf);
        let forwarder_handle = rt.spawn(input_forwarder(pcd_sub, filter_tx, reliable));
        let filter_handle = rt.spawn_blocking(move || {
            msg_processor(pcd_pub, filter_rx, gui_tx, filter);
        });
        futures::future::try_join(forwarder_handle, filter_handle)
    };

    // Start the GUI window if requested.
    let mut gui_ctx = gui_rx.map(|gui_rx| {
        let state = Gui::new(&config, lidar_tf, ground_tf, gui_rx);
        let mut window = Window::new(env! {"CARGO_BIN_NAME"});
        window.set_light(Light::StickToCamera);
        GuiContext { window, state }
    });

    // Main loop
    loop {
        // Update the GUI state
        if let Some(GuiContext { window, state }) = &mut gui_ctx {
            let ok = window.render_with_state(state);
            if !ok {
                break;
            }
        }

        // Spin the ROS runtime
        node.spin_once(Duration::from_millis(10));
    }

    // Wait for all workers to finish.
    rt.block_on(join)?;

    Ok(())
}

struct GuiContext {
    window: Window,
    state: Gui,
}

async fn wait_for_tf(
    mut tf_sub: impl Stream<Item = TransformStamped> + Unpin,
) -> Option<Transforms> {
    let mut lidar_tf = None;
    let mut ground_tf = None;

    loop {
        let tf = tf_sub.next().await?;

        match tf.child_frame_id.as_str() {
            "lidar" => {
                let tf = tf.to_na_isometry3();
                let tf = na::convert(tf);
                lidar_tf = Some(tf);
            }
            "gronud" => {
                let tf = tf.to_na_isometry3();
                let tf = na::convert(tf);
                ground_tf = Some(tf);
            }
            _ => {}
        }

        if let (Some(lidar_tf), Some(ground_tf)) = (lidar_tf, ground_tf) {
            break Some(Transforms {
                lidar_tf,
                ground_tf,
            });
        }
    }
}

async fn input_forwarder(
    mut pcd_sub: impl Stream<Item = PointCloud2> + Unpin,
    filter_tx: flume::Sender<PointCloud2>,
    reliable: bool,
) {
    while let Some(msg) = pcd_sub.next().await {
        if reliable {
            let ok = filter_tx.send_async(msg).await.is_ok();
            if !ok {
                break;
            }
        } else {
            match filter_tx.try_send(msg) {
                Ok(()) => {}
                Err(flume::TrySendError::Disconnected(_)) => break,
                Err(flume::TrySendError::Full(_)) => {}
            }
        }
    }
}

fn msg_processor(
    pcd_pub: Publisher<PointCloud2>,
    filter_rx: flume::Receiver<PointCloud2>,
    gui_tx: Option<flume::Sender<GuiMsg>>,
    filter: Filter,
) {
    while let Ok(in_msg) = filter_rx.recv() {
        let out_msg = match filter.process_msg(&in_msg) {
            Ok(out_msg) => out_msg,
            Err(err) => {
                log_error!(env!("CARGO_BIN_NAME"), "filter error: {err}");
                continue;
            }
        };

        let result = pcd_pub.publish(&out_msg);
        if let Err(err) = result {
            log_error!(env!("CARGO_BIN_NAME"), "publisher error: {err}");
            break;
        }

        if let Some(gui_tx) = &gui_tx {
            let gui_msg = GuiMsg {
                foreground_points: out_msg.to_na_point_vec().unwrap(),
                background_points: in_msg.to_na_point_vec().unwrap(),
            };

            let result = gui_tx.send(gui_msg);
            if let Err(err) = result {
                log_error!(env!("CARGO_BIN_NAME"), "GUI message error: {err}");
                break;
            }
        }
    }
}

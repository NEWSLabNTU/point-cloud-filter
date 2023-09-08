mod config;
mod filter;
mod gui;
mod params;
mod utils;

use crate::config::{Config,ConfigPath};
// use crate::filter::Filter;
use crate::gui::Gui;
// use anyhow::anyhow;
use anyhow::Result;
use clap::Parser;
use futures::Stream;
use futures::StreamExt;
use gui::GuiMsg;
use kiss3d::light::Light;
use kiss3d::window::Window;
use nalgebra as na;
// use params::Params;
use std::fs;
use std::path::PathBuf;
// use std::time::Duration;
// use tokio::runtime::Runtime;
// use std::env;
// use serde::{Deserialize, Serialize};
use serde_json;
use pcd_rs::{PcdDeserialize, Reader};
#[derive(PcdDeserialize)]
 pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: u32,
}
#[derive(Debug, Clone, Parser)]
/// Point cloud processor.
struct Opts {
    /// config file
    #[clap(long)]
    pub config: PathBuf,
}
#[derive(Debug)]
struct Transforms {
    pub lidar_tf: na::Isometry3<f32>,
    pub ground_tf: na::Isometry3<f32>,
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    config: String,
}
fn main() -> Result<()> {
    // TODO: Use clap to parse command line options
    // `cargo run -- --config filter.json5`
    // https://docs.rs/clap/latest/clap/
    let args=Args::parse();
    let s = fs::read_to_string(&PathBuf::from(&args.config))?;
    let config_path:ConfigPath = json5::from_str(&s)?;
    fs::create_dir_all(&config_path.output_path);   
    // Filter Config file format
    let filter_config:Config=  json5::from_str(&fs::read_to_string(&config_path.filter_config)?)?;
    let (gui_tx, gui_rx) = if filter_config.enable_gui {
        let (gui_tx, gui_rx) = flume::bounded(2);
        (Some(gui_tx), Some(gui_rx))
    } else {
        (None, None)
    };

    // TODO: Load lidar_tf.json and ground_tf.json
    let mut lidar_tf :serde_json::Value =  serde_json::from_str(&format!(r#" {{ "transforms" : {} }}"#,fs::read_to_string(&config_path.lidar_tf)?))?;
    lidar_tf = lidar_tf["transforms"][0]["transform"].clone();
    let lidar_tf_1 = na::Isometry3::from_parts(na::Translation3::new(
        lidar_tf["translation"][0].as_f64().unwrap() as f32,
        lidar_tf["translation"][1].as_f64().unwrap() as f32,
        lidar_tf["translation"][2].as_f64().unwrap() as f32,
    ),  na::Unit::new_normalize(na::Quaternion::new(
        lidar_tf["rotation"][3].as_f64().unwrap() as f32, // w
        lidar_tf["rotation"][0].as_f64().unwrap() as f32, // x
        lidar_tf["rotation"][1].as_f64().unwrap() as f32, // y
        lidar_tf["rotation"][2].as_f64().unwrap() as f32, // z
    )));

    let mut ground_tf :serde_json::Value =  serde_json::from_str(&format!(r#" {{ "transforms" : {} }}"#,fs::read_to_string(&config_path.ground_tf)?))?;
    ground_tf = ground_tf["transforms"][0]["transform"].clone();
    let ground_tf_1 = na::Isometry3::from_parts(na::Translation3::new(
        ground_tf["translation"][0].as_f64().unwrap() as f32,
        ground_tf["translation"][1].as_f64().unwrap() as f32,
        ground_tf["translation"][2].as_f64().unwrap() as f32,
    ), na::Unit::new_normalize(na::Quaternion::new(
        ground_tf["rotation"][3].as_f64().unwrap() as f32, // w
        ground_tf["rotation"][0].as_f64().unwrap() as f32, // x
        ground_tf["rotation"][1].as_f64().unwrap() as f32, // y
        ground_tf["rotation"][2].as_f64().unwrap() as f32, // z
    )));

    let transforms= Transforms{
        lidar_tf:lidar_tf_1,
        ground_tf:ground_tf_1
    };
   
    
    // Start the GUI window if requested.
    let mut gui_ctx = gui_rx.map(|gui_rx| {
        let state = Gui::new(&filter_config, lidar_tf_1, ground_tf_1, gui_rx);
        let mut window = Window::new(env! {"CARGO_BIN_NAME"});
        window.set_light(Light::StickToCamera);
        GuiContext { window, state }
    });

    // TODO: Open a directory of .pcd files
    let pcds = fs::read_dir(&config_path.data_path).unwrap();

    for pcd in pcds{
        // println!("{}", pcd.as_ref().unwrap().path().display());
        let reader = Reader::open(pcd.unwrap().path())?;
        let points: Result<Vec<Point>> = reader.collect();
        // println!("{} points found", points?.len());
        if let Some(GuiContext { window, state }) = &mut gui_ctx {
            let ok = window.render_with_state(state);
            if !ok {
                break;
            }
        }
        //TODO Start filter
        // let out_msg = match filter.process_msg(&in_msg) {
        //     Ok(out_msg) => out_msg,
        //     Err(err) => {
        //         log_error!(env!("CARGO_BIN_NAME"), "filter error: {err}");
        //         continue;
        //     }
        // };

        // let result = pcd_pub.publish(&out_msg);

        // if let Some(gui_tx) = &gui_tx {
        //     let gui_msg = GuiMsg {
        //         foreground_points: out_msg.to_na_point_vec().unwrap(),
        //         background_points: in_msg.to_na_point_vec().unwrap(),
        //     };

        //     let result = gui_tx.send(gui_msg);
        //     if let Err(err) = result {
        //         log_error!(env!("CARGO_BIN_NAME"), "GUI message error: {err}");
        //         break;
        //     }
        // }
        // TODO: Write filtered points to a .pcd file
        // https://docs.rs/pcd-rs/latest/pcd_rs/#writer-1

    }
    
    // let pcd_sub = node.subscribe::<PointCloud2>("~/input/point_cloud", qos.clone())?;
    // let tf_sub = node.subscribe::<TransformStamped>("/tf", QosProfile::default())?;
    // let pcd_pub = node.create_publisher::<PointCloud2>("~/output/point_cloud", qos)?;


    // // Main loop
    // // TODO: Load .pcd files one by one and call filter.process_msg each.
    // // Take msg_processor() for reference.

    // Wait for all workers to finish.
    rt.block_on(join)?;

    Ok(())
}

struct GuiContext {
    window: Window,
    state: Gui,
}

// fn msg_processor(
//     pcd_pub: Publisher<PointCloud2>,
//     filter_rx: flume::Receiver<PointCloud2>,
//     gui_tx: Option<flume::Sender<GuiMsg>>,
//     filter: Filter,
// ) {
//     while let Ok(in_msg) = filter_rx.recv() {
//         let out_msg = match filter.process_msg(&in_msg) {
//             Ok(out_msg) => out_msg,
//             Err(err) => {
//                 log_error!(env!("CARGO_BIN_NAME"), "filter error: {err}");
//                 continue;
//             }
//         };

//         let result = pcd_pub.publish(&out_msg);
//         if let Err(err) = result {
//             log_error!(env!("CARGO_BIN_NAME"), "publisher error: {err}");
//             break;
//         }

//         if let Some(gui_tx) = &gui_tx {
//             let gui_msg = GuiMsg {
//                 foreground_points: out_msg.to_na_point_vec().unwrap(),
//                 background_points: in_msg.to_na_point_vec().unwrap(),
//             };

//             let result = gui_tx.send(gui_msg);
//             if let Err(err) = result {
//                 log_error!(env!("CARGO_BIN_NAME"), "GUI message error: {err}");
//                 break;
//             }
//         }
//     }
// }

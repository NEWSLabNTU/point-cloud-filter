mod config;
mod filter;
mod gui;

use crate::{
    config::Config,
    gui::{Gui, GuiMsg},
};
use anyhow::Result;
use clap::Parser;
use filter::Filter;
use itertools::Itertools;
use kiss3d::{light::Light, window::Window};
use nalgebra_0_32 as na;
use pcd_rs::{DataKind, PcdSerialize, WriterInit};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Parser)]
/// Point cloud filtering tool.
struct Opts {
    /// Configuration file.
    #[clap(short = 'c', long)]
    pub config: PathBuf,

    /// The input directory containing .pcd files.
    #[clap(short = 'i', long)]
    pub input_dir: PathBuf,

    /// The output directory to store generated .pcd files.
    #[clap(short = 'o', long)]
    pub output_dir: PathBuf,

    /// Enable graphics user interface.
    #[clap(long)]
    pub gui: bool,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let config: Config = {
        let text = fs::read_to_string(&opts.config)?;
        json5::from_str(&text)?
    };

    let filter = Filter::new(&config);

    fs::create_dir_all(&opts.output_dir)?;
    // Filter Config file format

    let (gui_tx, gui_rx) = if opts.gui {
        let (gui_tx, gui_rx) = flume::bounded(2);
        (Some(gui_tx), Some(gui_rx))
    } else {
        (None, None)
    };

    // Start the GUI window if requested.
    let mut gui_ctx = gui_rx.map(|gui_rx| {
        let state = Gui::new(&config, gui_rx);
        let mut window = Window::new(env! {"CARGO_BIN_NAME"});
        window.set_light(Light::StickToCamera);
        GuiContext { window, state }
    });

    // TODO: Open a directory of .pcd files
    let pcd_files = fs::read_dir(&opts.output_dir)?
        .map(|entry| -> Result<_> {
            let entry = entry?;
            let path = entry.path();

            let Some(ext) = path.extension() else {
            return Ok(None);
        };

            if ext != ".pcd" {
                eprintln!("ignore file {}", path.display());
                return Ok(None);
            }

            Ok(Some(path))
        })
        .flatten_ok();

    for input_path in pcd_files {
        let input_path = input_path?;
        let reader = pcd_rs::DynReader::open(&input_path)?;

        let input_points: Vec<_> = reader
            .map(|record| -> Result<_> {
                let Some([x, y, z]) = record?.to_xyz() else {
                    return Ok(None)
                };
                let point = na::Point3::new(x, y, z);
                Ok(Some(point))
            })
            .flatten_ok()
            .try_collect()?;

        if input_points.is_empty() {
            continue;
        }

        if let Some(GuiContext { window, state }) = &mut gui_ctx {
            let ok = window.render_with_state(state);
            if !ok {
                break;
            }
        }

        let output_points = filter.process_msg(input_points.clone())?;

        if let Some(gui_tx) = &gui_tx {
            let gui_msg = GuiMsg {
                foreground_points: output_points.clone(),
                background_points: input_points,
            };

            let result = gui_tx.send(gui_msg);
            if let Err(err) = result {
                eprintln!("GUI message error: {err}");
                break;
            }
        }

        // Write filtered points to a .pcd file
        #[derive(PcdSerialize)]
        struct PcdPoint {
            pub x: f32,
            pub y: f32,
            pub z: f32,
        }

        let output_path = opts.output_dir.join(input_path.file_name().unwrap());

        let mut writer = WriterInit {
            width: output_points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: None,
        }
        .create(output_path)?;

        for point in output_points {
            let na::coordinates::XYZ { x, y, z } = *point;
            writer.push(&PcdPoint { x, y, z })?;
        }

        writer.finish()?;
    }

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

use crate::{config, message as msg};


pub async fn producer_future(
    stream: impl Stream<Item = Arc<msg::ProcessorMessage>> + Send + 'static,
    config: ArcConfig<config::Config>,
) -> Result<()> {
    let sink = config
        .dataflow
        .build_sender(env!("CARGO_PKG_NAME"))
        .await?
        .into_sink();
    let major_device = config.clone().map(|config| &config.major_device);

    stream
        .map(Ok)
        .try_par_map(2, move |msg| {
            let major_device = major_device.clone();
            move || fuse_points_and_construct_message(&*msg.msg, &*msg.points, &major_device)
        })
        .forward(sink)
        .await?;

    Ok(())
}

pub fn fuse_points_and_construct_message(
    in_msg: &protos::DataFrame,
    points: &[msg::ProcessedPoint],
    major_device: &DevicePath,
) -> Result<Vec<u8>> {
    let fused_points: Vec<_> = points
        .iter()
        .map(|point| {
            let [x, y, z]: [f64; 3] = point.point_in_major.into();
            let protos::LidarPoint {
                timestamp_ns,
                intensity,
                laser_id,
                distance,
                original_azimuth_angle,
                corrected_azimuth_angle,
                ..
            } = *point.orig_point;
            let is_active = point.is_active;

            protos::ProcessedPoint {
                position: protos::Point3D { x, y, z },
                timestamp_ns,
                intensity,
                device: point.device.borrow().into(),
                laser_id,
                is_active,
                distance,
                original_azimuth_angle,
                corrected_azimuth_angle,
                ..Default::default()
            }
        })
        .collect();

    let out_msg = protos::DataFrame {
        processed_points: protos::PointCloudProcess {
            major_device: major_device.borrow().into(),
            fused_points,
        }
        .into(),
        ..in_msg.clone()
    };
    let payload = out_msg.encode_to_vec();

    Ok(payload)
}

use anyhow::Result;
use generic_point_filter::{Config, Filter};
use itertools::Itertools;
use nalgebra::Point3;
use std::fs;

#[test]
fn simple_filter_test() -> Result<()> {
    // Construct the filter according to the config file.
    let text = fs::read_to_string("tests/simple.json5")?;
    let config: Config = json5::from_str(&text)?;
    let filter = Filter::new(&config);

    // Read points from the .pcd file
    let reader = pcd_rs::DynReader::open("tests/simple.pcd")?;
    let input_points: Vec<Point3<f32>> = reader
        .map(|record| -> Result<_> {
            let Some([x, y, z]) = record?.to_xyz() else {
                return Ok(None);
            };
            let point = Point3::new(x, y, z);
            Ok(Some(point))
        })
        .flatten_ok()
        .try_collect()?;

    // Run the filter
    let output_points = filter.process_msg(input_points.clone())?;
    assert!(output_points.len() < input_points.len());

    Ok(())
}

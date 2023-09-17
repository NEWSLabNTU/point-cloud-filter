use anyhow::Result;
use nalgebra::Point3;
use range_point_filter::FilterProgram;
use std::fs;

#[test]
fn box_filter_test() -> Result<()> {
    let config = fs::read_to_string("tests/box_filter.json5")?;
    let program: FilterProgram = json5::from_str(&config)?;

    let samples = [
        (Point3::new(3.0, 2.0, 2.5), Some(0.0), false),
        (Point3::new(3.0, 2.0, 2.5), Some(1.0), false),
        (Point3::new(3.0, 2.0, 1.0), Some(0.0), false),
        (Point3::new(3.0, 2.0, 1.0), Some(1.0), true),
        (Point3::new(3.0, 2.0, 1.0), Some(3.0), false),
        (Point3::new(1.49, 0.9, 2.0), Some(1.0), true),
        (Point3::new(1.51, 1.1, 2.0), Some(1.0), false),
    ];

    for (point, intensity, expect) in samples {
        assert_eq!(
            program.contains(&point, intensity),
            expect,
            "point={point} intensity={intensity:?}"
        );
    }

    Ok(())
}

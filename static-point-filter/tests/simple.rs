use background_point_filter::StatisticalStaticPointFilter;
use nalgebra::Point3;
use std::fs;

#[test]
fn simple_filter_test() {
    let config = fs::read_to_string("tests/simple.json").unwrap();
    let filter: StatisticalStaticPointFilter = serde_json::from_str(&config).unwrap();
    let point1 = Point3::origin();
    let point2 = Point3::new(-4.3, 0.0, 0.0);
    let point3 = Point3::new(100.0, 0.0, 0.0);

    assert!(filter.check_is_background(&point1));
    assert!(filter.check_is_background(&point2));
    assert!(filter.check_is_background(&point3));
    filter.step();

    for _ in 1..50 {
        assert!(filter.check_is_background(&point1));
        assert!(filter.check_is_background(&point2));
        assert!(filter.check_is_background(&point3));
        filter.step();
    }

    for _ in 50..65 {
        assert!(filter.check_is_background(&point1));
        assert!(filter.check_is_background(&point3));
        filter.step();
    }

    assert!(filter.check_is_background(&point1));
    assert!(!filter.check_is_background(&point2));
    assert!(filter.check_is_background(&point3));
}

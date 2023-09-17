# range-point-filter

Configurable point cloud filter written in Boolean expressions.

## Example

This library allows you to define multiple box ranges and combine them
using Boolean logic. Let's see the `config.json` example. It picks up
points inside the larger `box1`, excludes those within the smaller
`box2`, and keeps those points which intensity is between 0.5 and 2.5.

```json
{
    "program": "int * (box1 - box2)",
    "items": {
        "box1": {
            "type": "box",
            "xy": [3, 2],
            "wh": [6, 4],
            "z_min": 1,
            "z_max": 4,
            "azimuth_degrees": 0
        },
        "box2": {
            "type": "box",
            "xy": [3, 2],
            "wh": [2, 3],
            "z_min": 2,
            "z_max": 3,
            "azimuth_degrees": 90
        },
        "int" : {
            "type": "intensity",
            "min": 0.5,
            "max": 2.5
        }
    }
}
```

In the Rust program, load the filter using `serde`. The filter is
constructed out of box.

```rust
use range_point_filter::FilterProgram;

let config = std::fs::read_to_string("config.json")?;
let filter: FilterProgram = serde_json::from_str(&config)?;

let point = Point3::new(3.0, 2.0, 2.5);
let yes = filter.contains(&point, Some(intensity));
assert!(!yes);
```

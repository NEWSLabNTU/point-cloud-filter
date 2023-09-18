# static-point-filter

It implements a statistical static point filter, which consumes a
series of point cloud frames and counts point occurrences within each
voxel. It marks the voxel to be _static_ if it is occupied in majority
number of frames. Points within static voxels are dropped later on.

This library was contributed by Vicent Kuo, and then was revised by
Lin Hsiang-Jui.

## Usage

Configure the filter in this `example.json`.

```json
{
    "range": {
        "x_bound": [-4.5, 4.5],
        "y_bound": [-4.5, 4.5],
        "z_bound": [-4.5, 4.5]
    },
    "voxel_size": {
        "x_size": 1.0,
        "y_size": 1.0,
        "z_size": 1.0
    },
    "background_threshold": 0.8
}
```

The filter runs in a check-then-step cycle. It consumes points of each
frame and check if each point is static or not. Then, call `step()` to
update the internal statistics.

```rust
let config = fs::read_to_string("tests/simple.json").unwrap();
let filter: StatisticalStaticPointFilter = serde_json::from_str(&config).unwrap();

for input_points in frames {
    // input_points is Vec<Point3<f64>>

    // Check if each point is marked as static or not.
    // The point occurrence count increases in the mean time.
    let output_points: Vec<_> = input_points
        .into_iter()
        .map(|point| !filter.check_is_background(&point))
        .collect();
    
    // Update statistics
    filter.step();
}

```

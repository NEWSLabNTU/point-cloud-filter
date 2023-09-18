# point-filter-tool

A command-line tool that applies point filtering on .pcd point cloud
files.

## Usage

Prepare your .pcd files in one input directory and a configuration
file. An example configuration file can be found at
`config/config.json5`. Run this command to invoke this program.

```sh
cargo run --release -- \
    -c config/config.json5 \
    -i input_dir \
    -o output_dir
```

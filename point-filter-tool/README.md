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
    -o output_dir \
    --gui
```

建置config range_filter中的program的時候 最好寫成(Σsaved)-(Σfiltered) 用太多負號會爆炸
ex:"program": "(int1+int2+sl1+sl2+fl1+fl2+fl3)-(is1+is2+is3+is4+is5+bo1+bo2+bo3+bo4+tr1)" OK
"program": "int1+int2+sl1+sl2+fl1+fl2+fl3-is1-is2-is3-is4-is5-bo1-bo2-bo3-bo4-tr1" BOOM
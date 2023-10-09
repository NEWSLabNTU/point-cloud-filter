# generic-point-filter

This library provides the configuration and construction of a generic
point filter, which combines ground filter, range filter and
background filter.

## Usage

Add this section to `Cargo.toml` include this library in your Rust
project.

```toml
[dependencies.generic-point-filter]
git = "https://github.com/NEWSLabNTU/point-cloud-filter.git"
tag = "v0.1.0"
```

Construct the filter from a .json5 configuration file for example. The
detailed usage can be found at the [simple example](tests/simple.rs).

```rust
let text = fs::read_to_string("config.json5")?;
let config: Config = json5::from_str(&text)?;
let filter = Filter::new(&config);
```

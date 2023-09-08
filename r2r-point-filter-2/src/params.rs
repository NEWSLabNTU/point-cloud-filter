use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use r2r::Node;
use r2r::ParameterValue;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub enable_gui: bool,
    pub config: PathBuf,
}

impl Params {
    pub fn from_r2r_node(node: &Node) -> Result<Self> {
        let params = node
            .params
            .lock()
            .map_err(|_| anyhow!("node.params Mutex is poisoned."))?;

        let get_string = |key: &str| {
            let value = params
                .get(key)
                .ok_or_else(|| anyhow!("`{key}` parameter is not set"))?;
            let ParameterValue::String(value) = value else {
                bail!("`{key}` parameter must be a string");
            };
            Ok(value)
        };
        let get_bool = |key: &str| {
            let value = params
                .get(key)
                .ok_or_else(|| anyhow!("`{key}` parameter is not set"))?;
            let ParameterValue::Bool(value) = value else {
                bail!("`{key}` parameter must be a boolean value");
            };
            Ok(*value)
        };

        let config = PathBuf::from(get_string("config")?);
        let enable_gui = get_bool("enable_gui")?;

        Ok(Self { config, enable_gui })
    }
}

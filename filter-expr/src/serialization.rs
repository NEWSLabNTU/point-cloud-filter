use crate::parser::Program;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Program {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let text = format!("{self}");
        text.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Program {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        let lang =
            crate::parser::parse_str(&text).map_err(|err| D::Error::custom(format!("{err}")))?;
        Ok(lang)
    }
}

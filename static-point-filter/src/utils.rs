use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Serialize, Deserialize)]
struct SerializedBound<T>(pub [T; 2]);

impl<T> From<RangeInclusive<T>> for SerializedBound<T> {
    fn from(range: RangeInclusive<T>) -> Self {
        let (lhs, rhs) = range.into_inner();
        Self([lhs, rhs])
    }
}

impl<T> From<SerializedBound<T>> for RangeInclusive<T> {
    fn from(range: SerializedBound<T>) -> Self {
        let SerializedBound([lhs, rhs]) = range;
        lhs..=rhs
    }
}

pub mod serde_bound {
    use super::SerializedBound;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::ops::RangeInclusive;

    pub fn serialize<S, T>(bound: &RangeInclusive<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize + Clone,
        S: Serializer,
    {
        SerializedBound::from((*bound).clone()).serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<RangeInclusive<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let raw: SerializedBound<T> = Deserialize::deserialize(deserializer)?;
        let range: RangeInclusive<T> = raw.into();
        Ok(range)
    }
}

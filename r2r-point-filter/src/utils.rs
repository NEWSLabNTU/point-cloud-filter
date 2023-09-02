use serde::{Deserialize, Serialize};
use std::ops::{Bound, Bound::*};

fn unpack<T>(bound: &Bound<T>) -> (Option<&T>, Option<&T>) {
    match bound {
        Unbounded => (None, None),
        Included(val) => (None, Some(val)),
        Excluded(val) => (Some(val), None),
    }
}

fn pack<T>(bound: Option<T>, ibound: Option<T>) -> Option<Bound<T>> {
    let output = match (bound, ibound) {
        (None, None) => Unbounded,
        (Some(val), None) => Excluded(val),
        (None, Some(val)) => Included(val),
        (Some(_), Some(_)) => return None,
    };
    Some(output)
}

#[derive(Serialize, Deserialize)]
struct SerializedBound<T> {
    #[serde(rename = ">")]
    pub min: Option<T>,
    #[serde(rename = ">=")]
    pub imin: Option<T>,
    #[serde(rename = "<")]
    pub max: Option<T>,
    #[serde(rename = "<=")]
    pub imax: Option<T>,
}

impl<'a, T> SerializedBound<&'a T> {
    pub fn from_bound((lower, upper): &'a (Bound<T>, Bound<T>)) -> Self {
        let (min, imin) = unpack(lower);
        let (max, imax) = unpack(upper);

        SerializedBound {
            min,
            imin,
            max,
            imax,
        }
    }
}

impl<T> SerializedBound<T> {
    pub fn into_bound(self) -> Result<(Bound<T>, Bound<T>), &'static str> {
        let SerializedBound {
            min,
            imin,
            max,
            imax,
        } = self;

        let lower = pack(min, imin).ok_or("min and imin must not be both specified")?;
        let upper = pack(max, imax).ok_or("max and imax must not be both specified")?;

        Ok((lower, upper))
    }
}

pub mod serde_option_bound {
    use super::SerializedBound;
    use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
    use std::ops::Bound;

    pub fn serialize<S, T>(
        bound: &Option<(Bound<T>, Bound<T>)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        bound
            .as_ref()
            .map(|bound| SerializedBound::from_bound(bound))
            .serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<(Bound<T>, Bound<T>)>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let bound = Option::<SerializedBound<T>>::deserialize(deserializer)?
            .map(|raw| raw.into_bound())
            .transpose()
            .map_err(D::Error::custom)?;
        Ok(bound)
    }
}

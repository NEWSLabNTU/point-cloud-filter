pub use anyhow::{anyhow, bail, ensure, Error, Result};
pub use async_rt::{spawn, spawn_blocking};
pub use common_types as protos;
pub use common_types::serde_types::DevicePath;
pub use concurrent_slice::ConcurrentSlice as _;
pub use futures::{
    future::{self, FutureExt as _, TryFutureExt as _},
    sink::{Sink, SinkExt as _},
    stream,
    stream::{Stream, StreamExt as _, TryStreamExt as _},
};
pub use iterator_ext::IteratorExt as _;
pub use itertools::Itertools as _;
pub use log::{error, info, warn};
pub use nalgebra as na;
pub use noisy_float::prelude::*;
pub use ownref::ArcRefA;
pub use par_stream::prelude::*;
pub use prost::Message as _;
pub use std::{
    borrow::Borrow,
    cmp,
    collections::HashMap,
    ops::{Bound, Bound::*, RangeBounds as _},
    path::PathBuf,
    sync::Arc,
    thread,
    time::Duration,
};

pub type ArcMsg<T = protos::DataFrame> = ArcRefA<'static, protos::DataFrame, T>;
pub type ArcConfig<T = crate::config::Config> = ArcRefA<'static, crate::config::Config, T>;

mod config;
mod utils;

use config::Config;
use dashmap::DashMap;
use nalgebra::Point3;
use noisy_float::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{
    atomic::{AtomicU64, Ordering::*},
    RwLock,
};

#[derive(Debug)]
pub struct StaticPointFilter {
    config: Config,
    inner: RwLock<Inner>,
}

impl Clone for StaticPointFilter {
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap();
        Self {
            config: self.config.clone(),
            inner: RwLock::new(inner.clone()),
        }
    }
}

impl Serialize for StaticPointFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.config.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StaticPointFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = Config::deserialize(deserializer)?;
        Ok(Self::new(&config))
    }
}

#[derive(Debug, Clone)]
pub struct Inner {
    voxels: DashMap<[usize; 3], Voxel>,
    step: usize,
    mask: u64,
    threshold: u64,
}

#[derive(Debug, Default)]
struct Voxel {
    bits: AtomicU64,
    count: AtomicU64,
}

impl Clone for Voxel {
    fn clone(&self) -> Self {
        let Self { bits, count } = self;
        let bits = bits.load(Acquire);
        let count = count.load(Acquire);
        Self {
            bits: AtomicU64::new(bits),
            count: AtomicU64::new(count),
        }
    }
}

impl StaticPointFilter {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            inner: RwLock::new(Inner {
                voxels: DashMap::new(),
                step: 0,
                mask: 1,
                threshold: 0,
            }),
        }
    }

    pub fn step(&self) {
        let background_threshold = self.config.background_threshold.raw();
        let Inner {
            step,
            threshold,
            mask,
            voxels,
            ..
        } = &mut *self.inner.write().unwrap();

        *step += 1;
        *threshold = (*step as f64 * background_threshold).ceil() as u64;

        if *step % u64::BITS as usize == 0 {
            voxels.iter().for_each(|entry| {
                let Voxel { count, bits } = entry.value();
                let addition = bits.swap(0, Acquire).count_ones() as u64;
                count.fetch_add(addition, Release);
            });
            *mask = 1;
        } else {
            *mask <<= 1;
        }
    }

    pub fn check_is_background(&self, point: &Point3<f64>) -> bool {
        let Config {
            range:
                config::Range {
                    ref x_bound,
                    ref y_bound,
                    ref z_bound,
                },
            voxel_size:
                config::VoxelSize {
                    x_size,
                    y_size,
                    z_size,
                },
            ..
        } = self.config;
        let Inner {
            ref voxels,
            mask,
            threshold,
            ..
        } = *self.inner.read().unwrap();

        let [px, py, pz]: [f64; 3] = (*point).into();

        if !(x_bound.contains(&r64(px)) && y_bound.contains(&r64(py)) && z_bound.contains(&r64(pz)))
        {
            return true;
        }

        let index = {
            let x_idx = ((px - x_bound.start().raw()) / x_size.raw()).floor() as usize;
            let y_idx = ((py - y_bound.start().raw()) / y_size.raw()).floor() as usize;
            let z_idx = ((pz - z_bound.start().raw()) / z_size.raw()).floor() as usize;
            [x_idx, y_idx, z_idx]
        };

        let entry = voxels.entry(index).or_insert_with(Voxel::default);
        let Voxel { count, bits } = entry.value();
        let bits = bits.fetch_or(mask, Relaxed);
        let total_count = count.load(Relaxed) + bits.count_ones() as u64;

        total_count >= threshold
    }
}

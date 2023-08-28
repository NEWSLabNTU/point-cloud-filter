mod config;
mod utils;
pub use config::*;

use dashmap::DashMap;
use nalgebra as na;
use noisy_float::prelude::*;
use std::sync::{
    atomic::{AtomicU64, Ordering::*},
    RwLock,
};

#[derive(Debug)]
pub struct BackgroundPointFilter {
    inner: RwLock<Inner>,
}

#[derive(Debug)]
pub struct Inner {
    config: config::Config,
    voxels: DashMap<(usize, usize, usize), Voxel>,
    step: usize,
    mask: u64,
    threshold: u64,
}

#[derive(Debug, Default)]
struct Voxel {
    bits: AtomicU64,
    count: AtomicU64,
}

impl BackgroundPointFilter {
    pub fn new(config: &config::Config) -> Self {
        Self {
            inner: RwLock::new(Inner {
                config: config.clone(),
                voxels: DashMap::new(),
                step: 0,
                mask: 1,
                threshold: 0,
            }),
        }
    }

    pub fn step(&self) {
        let Inner {
            step,
            threshold,
            config,
            mask,
            voxels,
            ..
        } = &mut *self.inner.write().unwrap();

        *step += 1;
        *threshold = (*step as f64 * config.background_threshold.raw()).ceil() as u64;

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

    pub fn check_is_background(&self, point: &na::Point3<f64>) -> bool {
        let Inner {
            config:
                config::Config {
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
                },
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

        let x_idx = ((px - x_bound.start().raw()) / x_size.raw()).floor() as usize;
        let y_idx = ((py - y_bound.start().raw()) / y_size.raw()).floor() as usize;
        let z_idx = ((pz - z_bound.start().raw()) / z_size.raw()).floor() as usize;
        let index = (x_idx, y_idx, z_idx);

        use dashmap::mapref::entry::Entry as E;
        let entry = match voxels.entry(index) {
            E::Occupied(entry) => entry.into_ref(),
            E::Vacant(entry) => entry.insert(Voxel::default()),
        };
        let Voxel { count, bits } = entry.value();
        let bits = bits.fetch_or(mask, Relaxed);
        let total_count = count.load(Relaxed) + bits.count_ones() as u64;

        total_count >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_filter_test() {
        let filter = config::Config {
            range: config::Range {
                x_bound: (r64(-4.5)..=r64(4.5)),
                y_bound: (r64(-4.5)..=r64(4.5)),
                z_bound: (r64(-4.5)..=r64(4.5)),
            },
            voxel_size: config::VoxelSize {
                x_size: r64(1.0),
                y_size: r64(1.0),
                z_size: r64(1.0),
            },
            background_threshold: r64(0.8),
        }
        .build();
        let point1 = na::Point3::origin();
        let point2 = na::Point3::new(-4.3, 0.0, 0.0);
        let point3 = na::Point3::new(100.0, 0.0, 0.0);

        assert!(filter.check_is_background(&point1));
        assert!(filter.check_is_background(&point2));
        assert!(filter.check_is_background(&point3));
        filter.step();

        for _ in 1..50 {
            assert!(filter.check_is_background(&point1));
            assert!(filter.check_is_background(&point2));
            assert!(filter.check_is_background(&point3));
            filter.step();
        }

        for _ in 50..65 {
            assert!(filter.check_is_background(&point1));
            assert!(filter.check_is_background(&point3));
            filter.step();
        }

        assert!(filter.check_is_background(&point1));
        assert!(!filter.check_is_background(&point2));
        assert!(filter.check_is_background(&point3));
    }
}

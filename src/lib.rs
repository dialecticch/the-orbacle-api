extern crate dotenv;
extern crate num_cpus;

pub mod analyzers;
pub mod api;
pub mod custom;
pub mod opensea;
pub mod profiles;
pub mod storage;
pub mod updater;

pub fn from_wei(f: f64) -> f64 {
    f / 10f64.powf(18f64)
}

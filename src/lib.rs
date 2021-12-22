extern crate dotenv;

pub mod analyzers;
pub mod opensea;
pub mod profiles;
pub mod storage;

pub fn from_wei(f: f64) -> f64 {
    f / 10f64.powf(18f64)
}

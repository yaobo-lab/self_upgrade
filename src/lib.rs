#![allow(unused_assignments)]
pub mod appbin;
pub mod config;
pub mod status;
pub mod types;
pub mod upgrade;
pub mod version;
pub use appbin::*;
pub use config::*;
pub use status::*;
pub use types::*;
pub use upgrade::*;

pub type ProgressCallback = Box<dyn Fn(UpgradeStatus, Option<i32>) + Send + Sync + 'static>;
pub type RollBackCallback = Box<dyn Fn(String) + Send + Sync + 'static>;

#[cfg(test)]
mod test;

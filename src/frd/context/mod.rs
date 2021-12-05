#[cfg(target_os = "horizon")]
mod ctr;
#[cfg(target_os = "horizon")]
pub use ::ctr::*;

#[cfg(not(target_os = "horizon"))]
mod mock;
#[cfg(not(target_os = "horizon"))]
pub use mock::*;

mod shared;
pub use shared::*;

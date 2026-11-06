//! Serial link to the board (no game knowledge).

mod device;
pub mod fake;
mod link;
mod transport;

pub use device::*;
pub use link::*;
pub use transport::*;

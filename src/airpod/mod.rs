pub mod airpods;
pub mod model;
pub mod packet;
pub mod side;

pub use airpods::{AirPods, VENDOR_ID, as_airpods};
pub use model::Model;
pub use side::Side;

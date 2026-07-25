//! Triangular grid coordinate system shared by the server (authority) and client (prediction,
//! rendering, touch picking). See `README.md` in this directory for the design rationale.

pub mod coords;
pub mod hexagon;
pub mod neighbors;

pub use coords::TriCoord;
pub use hexagon::{HexMapError, hexagon_tiles};
pub use neighbors::tiles_within;

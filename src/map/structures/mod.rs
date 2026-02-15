// src/map/structures/mod.rs
pub mod types;
pub mod rect_maze;
pub mod hex_maze;
pub mod circular_maze;
pub mod solid_rect;

pub use types::{Segment, BBox};
pub use rect_maze::{MazeAlgo, generate_rectangular_maze};
pub use hex_maze::generate_hex_maze;
pub use circular_maze::generate_circular_maze;
pub use solid_rect::generate_solid_rect;

//! All the necessary code to deal with all things the game-client needs.

extern crate cgmath;
extern crate glfw;

pub mod cmd_opts;
pub mod glfw_context;
pub mod scene;
pub mod geometry;
pub mod render;
pub mod freecam;





/// Generic `Event` representing a tick being computed.
pub struct TickEvent {}
impl super::router::event::Event for TickEvent {
	fn is_passive(&self) -> bool {false}
}

/// Generic `Event` representing a frame being drawn.
pub struct DrawEvent {
	pub window_size: (i32, i32),
	pub now: f64,
	pub interpolation: f32,
}
impl super::router::event::Event for DrawEvent {
	fn is_passive(&self) -> bool {false}
}

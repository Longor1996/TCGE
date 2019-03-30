extern crate cgmath;

pub mod cmd_opts;
pub mod glfw_context;
pub mod scene;
pub mod geometry;
pub mod render;
pub mod freecam;






pub struct TickEvent {}
impl super::router::event::Event for TickEvent {
	fn is_passive(&self) -> bool {false}
}

pub struct DrawEvent {
	pub window_size: (i32, i32),
	pub now: f64,
	pub interpolation: f32,
}
impl super::router::event::Event for DrawEvent {
	fn is_passive(&self) -> bool {false}
}

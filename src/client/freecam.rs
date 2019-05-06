//! This module defines a free-flying camera for debugging purposes.

// TODO: Integrate the debug-camera with the ECS once it comes up...
// TODO: The camera will need to be abstracted into a PlayerController...

use super::glfw::{Key, Action};
use super::settings;
use super::super::blocks;

#[allow(unused)]
use super::cgmath::{
	Vector3, Matrix4, Transform,
	InnerSpace, ElementWise
};

/// A simple free-flying camera for debugging purposes.
#[derive(Debug)]
pub struct Camera {
	pub active: bool,
	position: cgmath::Vector3<f32>,
	velocity: cgmath::Vector3<f32>,
	rotation: cgmath::Vector2<f32>,
	position_last: cgmath::Vector3<f32>,
	velocity_last: cgmath::Vector3<f32>,
	rotation_last: cgmath::Vector2<f32>,
	pub target: Option<blocks::BlockCoord>,
	pub block: Option<blocks::BlockState>,
	min_depth: f32,
	max_depth: f32,
	field_of_view: f32,
	mouse_sensivity: f32,
	invert_mouse: bool,
	move_speed: f32,
	pub crane: bool,
}

impl Camera {
	/// Creates a new camera with default settings.
	pub fn new() -> Camera {
		return Camera {
			active: true,
			position: cgmath::Vector3 { x: 0.0, y: 1.8, z: -3.0 },
			velocity: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
			rotation: cgmath::Vector2 { x: 0.0, y: 0.0 },
			position_last: cgmath::Vector3 { x: 0.0, y: 1.8, z: 0.0 },
			velocity_last: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
			rotation_last: cgmath::Vector2 { x: 0.0, y: 90.0 },
			target: None,
			block: None,
			min_depth: 0.1,
			max_depth: 1024.0,
			field_of_view: 90.0,
			mouse_sensivity: 0.25,
			invert_mouse: false,
			move_speed: 2.0 / 30.0,
			crane: true
		}
	}
	
	pub fn apply_settings(&mut self, settings: &settings::Settings) {
		let controls = match settings.table.get("freecam-controls") {
			Some(v) => match v.as_table() {
				Some(t) => t,
				None => {warn!("'freecam-controls' settings is not a table."); return}
			},
			None => {warn!("'freecam-controls' settings does not exist."); return}
		};
		
		if let Some(v) = controls.get("field-of-view") {
			self.field_of_view = v.as_float().expect("Value 'field-of-view' is not a float.") as f32;
		}
		
		if let Some(v) = controls.get("mouse-sensitivity") {
			self.mouse_sensivity = v.as_float().expect("Value 'mouse-sensitivity' is not a float.") as f32;
		}
		
		if let Some(v) = controls.get("mouse-inverted") {
			self.invert_mouse = v.as_bool().expect("Value 'mouse-inverted' is not a bool.");
		}
		
		if let Some(v) = controls.get("movement-speed") {
			self.move_speed = v.as_float().expect("Value 'movement-speed' is not a float.") as f32 / 30.0;
		}
		
		if let Some(v) = controls.get("crane") {
			self.crane = v.as_bool().expect("Value 'crane' is not a bool.") as bool;
		}
	}
	
	/// Returns the predicted position of the camera for a given interpolation factor.
	/// Pass in `0` to get the current position as updated in the last tick.
	pub fn get_position(&self, interpolation: f32) -> cgmath::Vector3<f32> {
		// simple movement prediction formula
		self.position + (self.velocity * interpolation)
	}
	
	/// Returns the predicted rotation of the camera for a given interpolation factor.
	/// Pass in `0` to get the current rotation as updated in the last tick.
	pub fn get_rotation(&self, _interpolation: f32) -> cgmath::Vector2<f32> {
		// TODO: movement prediction
		self.rotation // + ((self.rotation_last - self.rotation) * interpolation)
	}
	
	pub fn get_look_dir(&self, interpolation: f32) -> cgmath::Vector3<f32> {
		let rotation = self.get_rotation(interpolation);
		let pitch = cgmath::Deg(rotation.x);
		let yaw = cgmath::Deg(rotation.y);
		
		let mut camera = Matrix4::one();
		camera = camera * Matrix4::from_angle_y(yaw);
		camera = camera * Matrix4::from_angle_x(pitch);
		
		let forward = Vector3::new(0.0, 0.0, 1.0);
		let forward = Matrix4::transform_vector(&camera, forward);
		forward.normalize()
	}
	
	/// Given a viewport-size and an interpolation factor, compute the Projection-Matrix for this camera.
	pub fn projection(&self, _interpolation: f32, size: (i32, i32)) -> cgmath::Matrix4<f32> {
		let (width, height) = size;
		
		// Apply velocity to the FoV for speedy-effect
		// TODO: Add an option to disable this.
		let field_of_view = self.field_of_view + self.velocity.magnitude() * 23.42;
		
		let fov = cgmath::Rad::from(cgmath::Deg(field_of_view));
		
		// --- First compute the projection matrix...
		let perspective = cgmath::PerspectiveFov {
			fovy: fov,
			aspect: width as f32 / height as f32,
			near: self.min_depth,
			far: self.max_depth,
		};
		
		let perspective = Matrix4::from(perspective);
		perspective
	}
	
	/// Given an interpolation factor, compute the View-Matrix for this camera.
	///
	/// If `translation` is `false`, the camera position is ignored in the computation.
	pub fn transform(&self, interpolation: f32, translation: bool) -> cgmath::Matrix4<f32> {
		// --- Now compute the rotation matrix...
		let rotation = self.get_rotation(interpolation);
		let pitch = cgmath::Deg(rotation.x);
		let yaw = cgmath::Deg(rotation.y);
		
		let mut camera = Matrix4::one();
		camera = camera * Matrix4::from_angle_x(pitch);
		camera = camera * Matrix4::from_angle_y(yaw);
		
		// This line 'synchronizes' the coordinate systems of OpenGL and basic
		// trigonometry, such that sin(theta) means the same in both systems.
		camera = camera * Matrix4::from_nonuniform_scale(1.0, 1.0, -1.0);
		
		if translation {
			// And optionally compute and multiply with the translation matrix...
			camera = camera * Matrix4::from_translation(-self.get_position(interpolation));
		}
		
		// return view matrix
		camera
	}
	
	/// Updates the camera rotation by adding the given pitch/yaw euler-deltas.
	pub fn update_rotation(&mut self, yaw: f32, pitch: f32) {
		self.rotation_last.clone_from(&self.rotation);
		
		if !self.active {
			return;
		}
		
		let pitch = if self.invert_mouse { -pitch } else { pitch };
		
		self.rotation.x += pitch * self.mouse_sensivity;
		self.rotation.x = clamp(self.rotation.x, -90.0, 90.0);
		
		self.rotation.y += yaw * self.mouse_sensivity;
		self.rotation.y = wrap(self.rotation.y, 360.0);
	}
	
	/// Updates the camera position by querying key-states and changing the velocity accordingly.
	pub fn update_movement(&mut self, window: &glfw::Window) {
		self.position_last.clone_from(&self.position);
		self.velocity_last.clone_from(&self.velocity);
		
		if !self.active {
			return;
		}
		
		let mut move_speed = self.move_speed;
		
		// --- Apply speed multiplier?
		if window.get_key(Key::LeftShift) == Action::Press {
			move_speed *= 5.0;
		}
		
		// --- Construct velocity vector...
		let yaw = cgmath::Deg(self.rotation.y);
		let mut mat = Matrix4::from_angle_y(yaw);
		
		// Fetch the input statuses and convert them to 0/1...
		let forwards = (window.get_key(Key::W) == Action::Press) as i8;
		let backwards = (window.get_key(Key::S) == Action::Press) as i8;
		let strafe_left = (window.get_key(Key::A) == Action::Press) as i8;
		let strafe_right = (window.get_key(Key::D) == Action::Press) as i8;
		
		let mut direction = Vector3::new(0.0, 0.0, 0.0);
		
		// ...then build a direction vector from them.
		// - If neither are active, the result is 0.
		// - If only 'forwards'  is active, the result is +1.
		// - If only 'backwards' is active, the result is -1.
		// - If both are active, cancelling each other out, the result is 0.
		direction.z += (forwards - backwards) as f32;
		direction.x += (strafe_right - strafe_left) as f32;
		
		// crane or drone mode for y axis
		if self.crane {
			// CRANE: The camera pitch does not affect planar movement.
			let up = (window.get_key(Key::Space) == Action::Press) as i8;
			let down = (window.get_key(Key::LeftControl) == Action::Press) as i8;
			direction.y += (up - down) as f32;
		}
		else {
			// DRONE: The camera pitch tilts the plane of movement.
			let pitch = cgmath::Deg(self.rotation.x);
			mat = mat * Matrix4::from_angle_x(pitch);
		}
		
		// Ensure that the vector has a magnitude of 1 (equal in all directions)
		direction.normalize();
		
		// Transform the new velocity vector into world-space...
		let direction = Matrix4::transform_vector(&mat, direction);
		
		// ...and add it to the existing velocity vector.
		self.velocity += direction * move_speed;
		
		// Apply velocity
		self.position += self.velocity;
		
		// Friction
		self.velocity *= 0.75;
	}
}

impl std::fmt::Display for Camera {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "Camera({}-mode) [x: {}, y: {}, z: {}, pitch: {}, yaw: {} ] & LastCamera [x: {}, y: {}, z: {}, pitch: {}, yaw: {} ]",
			if self.crane { "crane" } else { "drone" },
			self.position.x,
			self.position.y,
			self.position.z,
			self.rotation.x,
			self.rotation.y,
			self.position_last.x,
			self.position_last.y,
			self.position_last.z,
			self.rotation_last.x,
			self.rotation_last.y,
		)
	}
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
	if x < min { return min; }
	if x > max { return max; }
	x
}

fn wrap(mut x: f32, r: f32) -> f32 {
	while x < 0.0 {
		x += r;
	}
	while x > r {
		x -= r;
	}
	
	x
}
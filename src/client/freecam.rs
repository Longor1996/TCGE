//! This module defines a free-flying camera for debugging purposes.

// TODO: Integrate the debug-camera with the ECS once it comes up...
// TODO: The camera will need to be abstracted into a PlayerController...

use super::glfw::{Key, Action};

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
	min_depth: f32,
	max_depth: f32,
	field_of_view: f32,
	mouse_sensivity: f32,
	invert_mouse: bool,
	move_speed: f32,
}

impl Camera {
	/// Creates a new camera.
	///
	/// TODO: Add some parameters.
	pub fn new() -> Camera {
		return Camera {
			active: true,
			position: cgmath::Vector3 { x: 0.0, y: 1.8, z: -3.0 },
			velocity: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
			rotation: cgmath::Vector2 { x: 0.0, y: 0.0 },
			position_last: cgmath::Vector3 { x: 0.0, y: 1.8, z: 0.0 },
			velocity_last: cgmath::Vector3 { x: 0.0, y: 0.0, z: 0.0 },
			rotation_last: cgmath::Vector2 { x: 0.0, y: 90.0 },
			min_depth: 0.1,
			max_depth: 1024.0,
			field_of_view: 90.0,
			mouse_sensivity: 0.25,
			invert_mouse: false,
			move_speed: 2.0 / 30.0,
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
	
	/// Given a viewport-size and an interpolation factor, compute the View-Projection-Matrix for this camera.
	///
	/// If `translation` is `false`, the camera position is ignored in the computation.
	pub fn transform(&self, size: (i32, i32), interpolation: f32, translation: bool) -> cgmath::Matrix4<f32> {
		let (width, height) = size;
		let fov = cgmath::Rad::from(cgmath::Deg(self.field_of_view));
		
		// --- First compute the projection matrix...
		let perspective = cgmath::PerspectiveFov {
			fovy: fov,
			aspect: width as f32 / height as f32,
			near: self.min_depth,
			far: self.max_depth,
		};
		
		let perspective = Matrix4::from(perspective);
		
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
		
		// return multiplied matrix
		perspective * camera
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
			move_speed = move_speed * 5.0;
		}
		
		// --- Move downwards?
		if window.get_key(Key::LeftControl) == Action::Press {
			self.velocity += Vector3::new(0.0, -1.0, 0.0) * move_speed;
		}
		
		// --- Move upwards?
		if window.get_key(Key::Space) == Action::Press {
			self.velocity += Vector3::new(0.0, 1.0, 0.0) * move_speed;
		}
		
		let yaw = cgmath::Deg(self.rotation.y);
		let mat = Matrix4::from_angle_y(yaw);
		
		// --- Move forwards?
		let forward = Vector3::new(0.0, 0.0, 1.0);
		let forward = Matrix4::transform_vector(&mat, forward);
		if window.get_key(Key::W) == Action::Press {
			self.velocity += forward * move_speed;
		}
		
		// --- Move backwards?
		let backward = Vector3::new(0.0, 0.0, -1.0);
		let backward = Matrix4::transform_vector(&mat, backward);
		if window.get_key(Key::S) == Action::Press {
			self.velocity += backward * move_speed;
		}
		
		// --- Move sideways to the left?
		let left = Vector3::new(-1.0, 0.0, 0.0);
		let left = Matrix4::transform_vector(&mat, left);
		if window.get_key(Key::A) == Action::Press {
			self.velocity += left * move_speed;
		}
		
		// --- Move sideways to the right?
		let right = Vector3::new(1.0, 0.0, 0.0);
		let right = Matrix4::transform_vector(&mat, right);
		if window.get_key(Key::D) == Action::Press {
			self.velocity += right * move_speed;
		}
		
		// Apply velocity
		self.position = self.position + self.velocity;
		
		// Reduce velocity
		self.velocity = self.velocity * 0.5;
	}
}

impl std::fmt::Display for Camera {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "Camera [x: {}, y: {}, z: {}, pitch: {}, yaw: {} ] & LastCamera [x: {}, y: {}, z: {}, pitch: {}, yaw: {} ]",
			self.position.x,
			self.position.y,
			self.position.z,
			self.rotation.x,
			self.rotation.y,
			self.position_last.x,
			self.position_last.y,
			self.position_last.z,
			self.rotation_last.x,
			self.rotation_last.y
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
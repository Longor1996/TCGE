/*
	This file defines a free-flying camera for debugging purposes.
	
	TODO: Integrate the debug-camera with the ECS once it comes up.
*/

extern crate glfw;
use self::glfw::{Key, Action};

use super::cgmath::{
	Vector3, Matrix4, Transform,
	InnerSpace, ElementWise
};

// TODO: Camera needs PlayerController/ClientInput...
#[derive(Debug)]
pub struct Camera {
	pub position: cgmath::Vector3<f32>,
	pub velocity: cgmath::Vector3<f32>,
	pub rotation: cgmath::Vector2<f32>,
	pub position_last: cgmath::Vector3<f32>,
	pub velocity_last: cgmath::Vector3<f32>,
	pub rotation_last: cgmath::Vector2<f32>,
}

impl Camera {
	pub fn new() -> Camera {
		return Camera {
			position: cgmath::Vector3 {x: 0.0, y: 1.8, z: 0.0},
			velocity: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
			rotation: cgmath::Vector2 {x: 0.0, y: 90.0},
			position_last: cgmath::Vector3 {x: 0.0, y: 1.8, z: 0.0},
			velocity_last: cgmath::Vector3 {x: 0.0, y: 0.0, z: 0.0},
			rotation_last: cgmath::Vector2 {x: 0.0, y: 90.0}
		}
	}
	
	pub fn transform(&self, size: (i32,i32), interpolation: f32, translation: bool ) -> cgmath::Matrix4<f32> {
		let (width, height) = size;
		let fov = cgmath::Rad::from(cgmath::Deg(90.0));
		
		let perspective = cgmath::PerspectiveFov {
			fovy: fov,
			aspect: width as f32 / height as f32,
			near: 0.1, far: 1024.0
		};
		
		let perspective = Matrix4::from(perspective);
		
		// this next section can most certainly be written with less code...
		let mut camera = Matrix4::new(
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0
		);
		
		let pitch = cgmath::Deg(self.rotation.x);
		let yaw = cgmath::Deg(self.rotation.y);
		
		camera = camera * Matrix4::from_angle_x(pitch);
		camera = camera * Matrix4::from_angle_y(yaw);
		camera = camera * Matrix4::from_nonuniform_scale(1.0,1.0,-1.0);
		
		if translation {
			// simple movement prediction formula
			let pos = self.position + (self.velocity * interpolation);
			camera = camera * Matrix4::from_translation(-pos);
		}
		
		// return multiplied matrix
		perspective * camera
	}
	
	pub fn update_rotation(&mut self, yaw: f32, pitch: f32) {
		self.rotation_last.clone_from(& self.rotation);
		
		let mouse_sensivity = 0.5;
		
		self.rotation.x += pitch * mouse_sensivity;
		self.rotation.x = clamp(self.rotation.x, -90.0, 90.0);
		
		self.rotation.y += yaw * mouse_sensivity;
		self.rotation.y = wrap(self.rotation.y , 360.0);
	}
	
	pub fn update_movement(&mut self, window: & glfw::Window) {
		
		self.position_last.clone_from(& self.position);
		self.velocity_last.clone_from(& self.velocity);
		
		let mut move_speed = 0.5;
		
		if window.get_key(Key::LeftShift) == Action::Press {
			move_speed = move_speed * 4.0;
		}
		
		if window.get_key(Key::LeftControl) == Action::Press {
			self.position += Vector3::new(0.0, -1.0, 0.0) * move_speed;
		}
		if window.get_key(Key::Space) == Action::Press {
			self.position += Vector3::new(0.0, 1.0, 0.0) * move_speed;
		}
		
		let yaw = cgmath::Deg(self.rotation.y);
		let mat = Matrix4::from_angle_y(yaw);
		
		let forward = Vector3::new(0.0, 0.0, 1.0);
		let forward = Matrix4::transform_vector(&mat, forward);
		if window.get_key(Key::W) == Action::Press {
			self.position += forward * move_speed;
		}
		
		let backward = Vector3::new(0.0, 0.0, -1.0);
		let backward = Matrix4::transform_vector(&mat, backward);
		if window.get_key(Key::S) == Action::Press {
			self.position += backward * move_speed;
		}
		
		let left = Vector3::new(-1.0, 0.0, 0.0);
		let left = Matrix4::transform_vector(&mat, left);
		if window.get_key(Key::A) == Action::Press {
			self.position += left * move_speed;
		}
		
		let right = Vector3::new(1.0, 0.0, 0.0);
		let right = Matrix4::transform_vector(&mat, right);
		if window.get_key(Key::D) == Action::Press {
			self.position += right * move_speed;
		}
	}
}

impl std::fmt::Display for Camera {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "Camera [x: {}, z: {}, pitch: {}, yaw: {} ] & LastCamera [x: {}, z: {}, pitch: {}, yaw: {} ]",
		       self.position.x,
		       self.position.z,
		       self.rotation.x,
		       self.rotation.y,
		       self.position_last.x,
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
use glfw::{Key, Action};
use super::aabb::AxisAlignedBoundingBox;
use glfw::GamepadAxis::AxisLeftTrigger;
use nalgebra::{DimAdd, DimMul};
use std::ops::Add;

pub struct Freecam {
	pub active: bool,
	shape_radius: f32,
	shape_extent: f32,
	position: nalgebra_glm::Vec3,
	velocity: nalgebra_glm::Vec3,
	impulse: nalgebra_glm::Vec3,
	rotation: nalgebra_glm::Vec2,
	position_last: nalgebra_glm::Vec3,
	velocity_last: nalgebra_glm::Vec3,
	rotation_last: nalgebra_glm::Vec2,
	pub target: Option<blocks::BlockCoord>,
	min_depth: f32,
	max_depth: f32,
	field_of_view: f32,
	fov_vel_effect: bool,
	mouse_sensivity: f32,
	invert_mouse: bool,
	move_speed: f32,
	pub crane: bool,
	pub gravity: bool,
	const_gravity_power: f32,
	const_gravity_limit: f32,
	const_air_friction: f32,
	const_jump_power: f32,
	const_impulse_limit: f32,
}

impl Freecam {
	/// Creates a new camera with default settings.
	pub fn new() -> Self {
		Self {
			active: true,
			shape_radius: 0.3,
			shape_extent: 1.8/2.0,
			position: nalgebra_glm::Vec3::new(4.0, 3.0, -8.0),
			velocity: nalgebra_glm::Vec3::new ( 0.0, 0.0, 0.0 ),
			impulse: nalgebra_glm::Vec3::new ( 0.0, 0.0, 0.0 ),
			rotation: nalgebra_glm::Vec2::new (0.0, 0.0 ),
			position_last: nalgebra_glm::Vec3::new ( 0.0, 1.8, 0.0 ),
			velocity_last: nalgebra_glm::Vec3::new ( 0.0, 0.0, 0.0 ),
			rotation_last: nalgebra_glm::Vec2::new ( 0.0, 90.0 ),
			target: None,
			min_depth: 0.1,
			max_depth: 4096.0,
			field_of_view: 90.0,
			fov_vel_effect: false,
			mouse_sensivity: 0.0625,
			invert_mouse: false,
			move_speed: 0.5,
			crane: true,
			gravity: true,
			const_gravity_power: 0.0012,
			const_gravity_limit: 0.999,
			const_air_friction: 0.5,
			const_jump_power: 0.25,
			const_impulse_limit: 0.75,
		}
	}
	
	pub fn config(&mut self, table: &toml::value::Table) {
		
		info!("Applying [freecam] config...");
		
		if let Some(x) = table.get("position") {
			let pos = x.as_array().expect("position must be a table");
			self.position.x = pos.get(0).expect("x-component").as_float().expect("x-component not a float") as f32;
			self.position.y = pos.get(1).expect("y-component").as_float().expect("y-component not a float") as f32;
			self.position.z = pos.get(2).expect("z-component").as_float().expect("z-component not a float") as f32;
		} else {
			warn!("Missing key: position = [{:.2}, {:.2}, {:.2}]", self.position.x, self.position.y, self.position.z);
		}
		
		if let Some(x) = table.get("field_of_view") {
			self.field_of_view = x.as_float().unwrap_or(self.field_of_view as f64) as f32;
		} else {
			warn!("Missing key: field_of_view = {}", self.field_of_view);
		}
		
		if let Some(x) = table.get("mouse_sensivity") {
			self.mouse_sensivity = x.as_float().unwrap_or(self.mouse_sensivity as f64) as f32;
		} else {
			warn!("Missing key: mouse_sensivity = {}", self.mouse_sensivity);
		}
		
		if let Some(x) = table.get("move_speed") {
			self.move_speed = x.as_float().unwrap_or(self.move_speed as f64) as f32;
		} else {
			warn!("Missing key: move_speed = {}", self.move_speed);
		}
		
		if let Some(x) = table.get("jump_power") {
			self.const_jump_power = x.as_float().unwrap_or(self.const_jump_power as f64) as f32;
		} else {
			warn!("Missing key: jump_power = {}", self.const_jump_power);
		}
		
		if let Some(x) = table.get("gravity_power") {
			self.const_gravity_power = x.as_float().unwrap_or(self.const_gravity_power as f64) as f32;
		} else {
			warn!("Missing key: gravity_power = {}", self.const_gravity_power);
		}
		
		if let Some(x) = table.get("gravity_limit") {
			self.const_gravity_limit = x.as_float().unwrap_or(self.const_gravity_limit as f64) as f32;
		} else {
			warn!("Missing key: gravity_limit = {}", self.const_gravity_limit);
		}
		
		if let Some(x) = table.get("air_friction") {
			self.const_air_friction = x.as_float().unwrap_or(self.const_air_friction as f64) as f32;
		} else {
			warn!("Missing key: air_friction = {}", self.const_air_friction);
		}
	}
	
	/// Returns the predicted position of the camera for a given interpolation factor.
	/// Pass in `0` to get the current position as updated in the last tick.
	pub fn get_position(&self, interpolation: f32) -> nalgebra_glm::Vec3 {
		// simple movement prediction formula
		&self.position + (&self.velocity * interpolation)
	}
	
	pub fn get_velocity(&self, interpolation: f32) -> nalgebra_glm::Vec3 {
		// simple movement prediction formula
		&self.velocity * interpolation
	}
	
	/// Returns the predicted rotation of the camera for a given interpolation factor.
	/// Pass in `0` to get the current rotation as updated in the last tick.
	pub fn get_rotation_euler(&self, _interpolation: f32) -> nalgebra_glm::Vec2 {
		// TODO: movement prediction
		&self.rotation * 1.0 // + ((self.rotation_last - self.rotation) * interpolation)
	}
	
	pub fn get_eye_offset(&self, _interpolation: f32) -> nalgebra_glm::Vec3 {
		nalgebra_glm::Vec3::new (0.0, self.shape_extent*0.75, 0.0)
	}
	
	pub fn get_look_dir(&self, interpolation: f32) -> nalgebra_glm::Vec3 {
		let rotation = self.get_rotation_euler(interpolation);
		let pitch = rotation.x;
		let yaw = rotation.y;
		
		let mut camera = nalgebra_glm::Mat4::identity();
		
		camera = nalgebra_glm::rotate_y(&camera, yaw.to_radians());
		camera = nalgebra_glm::rotate_x(&camera, pitch.to_radians());
		
		let forward = nalgebra_glm::Vec3::new(0.0, 0.0, 1.0);
		
		//let forward = camera * forward;
		let forward: nalgebra_glm::Vec3 = camera.transform_vector(&forward);
		
		nalgebra_glm::normalize(&forward)
	}
	
	pub fn get_block_raytrace(&self, len: f32, interpolation: f32) -> blocks::BlockRaycast {
		let src = self.get_position(interpolation);
		let src = src + self.get_eye_offset(interpolation);
		let src = (src.x, src.y, src.z);
		
		let dir = self.get_look_dir(interpolation);
		let dir = (dir.x, dir.y, dir.z);
		
		blocks::BlockRaycast::new_from_src_dir_len(src, dir, len)
	}
	
	/// Updates the camera rotation by adding the given pitch/yaw euler-deltas.
	pub fn update_rotation(&mut self, yaw: f32, pitch: f32) -> bool {
		self.rotation_last.clone_from(&self.rotation);
		
		if !self.active {
			return false;
		}
		
		let pitch = if self.invert_mouse { -pitch } else { pitch };
		
		self.rotation.x += pitch * self.mouse_sensivity;
		self.rotation.x = clamp(self.rotation.x, -90.0, 90.0);
		
		self.rotation.y += yaw * self.mouse_sensivity;
		self.rotation.y = wrap(self.rotation.y, 360.0);
		true
	}
	
	/// Updates the camera position by querying key-states and changing the velocity accordingly.
	pub fn update_movement(&mut self, window: &glfw::Window, delta: f32, chunks: &super::ChunkStorage) {
		self.position_last.clone_from(&self.position);
		self.velocity_last.clone_from(&self.velocity);
		
		if !self.active {
			return;
		}
		
		let mut move_speed = self.move_speed * delta;
		
		// --- Apply speed multiplier?
		if window.get_key(Key::LeftShift) == Action::Press {
			move_speed *= 5.0;
		}
		
		// --- Construct velocity vector...
		let mut mat = nalgebra_glm::rotate_y(&nalgebra_glm::identity(), self.rotation.y.to_radians());
		
		// Fetch the input statuses and convert them to 0/1...
		let forwards = (window.get_key(Key::W) == Action::Press) as i8;
		let backwards = (window.get_key(Key::S) == Action::Press) as i8;
		let strafe_left = (window.get_key(Key::A) == Action::Press) as i8;
		let strafe_right = (window.get_key(Key::D) == Action::Press) as i8;
		
		let mut direction = nalgebra_glm::Vec3::new(0.0, 0.0, 0.0);
		
		// ...then build a direction vector from them.
		// - If neither are active, the result is 0.
		// - If only 'forwards'  is active, the result is +1.
		// - If only 'backwards' is active, the result is -1.
		// - If both are active, cancelling each other out, the result is 0.
		direction.z += (forwards - backwards) as f32;
		direction.x += (strafe_right - strafe_left) as f32;
		
		// crane or drone mode for y axis
		if self.crane || self.gravity {
			// CRANE: The camera pitch does not affect planar movement.
			let up = (window.get_key(Key::Space) == Action::Press) as i8;
			let down = (window.get_key(Key::LeftControl) == Action::Press) as i8;
			direction.y += (up - down) as f32;
		}
		else {
			// DRONE: The camera pitch tilts the plane of movement.
			let pitch = self.rotation.x.to_radians();
			mat = nalgebra_glm::rotate_x(&mat, pitch);
		}
		
		if self.gravity {
			direction.y = 0.0;
		}
		
		// Ensure that the vector has a magnitude of 1 (equal in all directions)
		direction.normalize();
		
		// Transform the new velocity vector into world-space...
		
		let direction = mat.transform_vector(&direction);
		
		// ...and add it to the existing velocity vector.
		self.velocity += direction * move_speed;
		
		if self.gravity {
			// Apply Gravity
			self.velocity.y -= self.const_gravity_power;
			self.velocity.y *= if self.velocity.y < 0.0 {self.const_gravity_limit} else {self.const_air_friction};
		}
		
		self.velocity += &self.impulse;
		
		// Now do collision checks
		let player_box = AxisAlignedBoundingBox::from_position_radius_height(self.position, self.shape_radius, self.shape_extent);
		let mut block_boxes = vec![];
		
		let bpx = self.position.x.floor() as i32;
		let bpy = self.position.y.floor() as i32;
		let bpz = self.position.z.floor() as i32;
		let rad = 3;
		
		for y in (bpy-rad)..(bpy+rad) {
			for z in (bpz-rad)..(bpz+rad) {
				for x in (bpx-rad)..(bpx+rad) {
					let pos = blocks::BlockCoord::new(x, y, z);
					if let Some(block) = chunks.get_block(&pos) {
						// only match NOT AIR for now
						if block.id.raw() != 0 {
							let vec_pos = nalgebra_glm::Vec3::new(x as f32, y as f32, z as f32);
							let box_ext = nalgebra_glm::Vec3::new(1.0, 1.0, 1.0);
							let block_box = AxisAlignedBoundingBox::from_position_size(vec_pos, box_ext);
							block_boxes.push(block_box);
						}
					}
				}
			}
		}
		
		let is_falling = self.velocity.y < 0.0;
		
		/*
		let mut hits: Vec<_> = block_boxes.iter()
			.filter_map(|block_box| AxisAlignedBoundingBox::sweep_self(&player_box, &self.velocity, &block_box))
			.collect();
		
		hits.sort_by(|a, b| {a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)});
		
		
		println!("hits: {}", hits.len());
		
		for (time, over, velo, norm) in hits {
			println!("sweep: {:.2}, {:?}, {:?}, {:?}", time, over, velo, norm);
			
			if velo.x != 0.0 || velo.y != 0.0 || velo.z != 0.0 {
				if norm.x != 0.0 {
					self.velocity.x = 0.0;
				}
				if norm.y != 0.0 {
					self.velocity.y = 0.0;
				}
				if norm.z != 0.0 {
					self.velocity.z = 0.0;
				}
			}
			
			// self.velocity = *velo;
		}
		*/
		
		// TODO: Read https://github.com/oniietzschan/bump-3dpd/blob/master/bump-3dpd.lua
		// TODO: Read https://github.com/andyhall/voxel-aabb-sweep/blob/master/index.js
		// TODO: Read https://flipcode.com/archives/3D_Pong_Collision_Response.shtml
		
		/*
		println!("hits: {}", hits.len());
		for (time, velo, norm) in hits {
			
			println!("sweep: {:.2}, {:?}, {:?}", time, velo, norm);
			self.velocity = velo;
		}
		*/
		
		for block_box in block_boxes.iter() {
			self.velocity.y = block_box.intersection_y(&player_box, self.velocity.y);
		}
		
		for block_box in block_boxes.iter() {
			self.velocity.x = block_box.intersection_x(&player_box, self.velocity.x);
		}
		
		for block_box in block_boxes.iter() {
			self.velocity.z = block_box.intersection_z(&player_box, self.velocity.z);
		}
		
		let is_on_ground = self.velocity.y == 0.0 && is_falling;
		
		// Apply velocity
		self.position += &self.velocity;
		
		// Reduce impulse
		self.impulse *= self.const_impulse_limit;
		
		let air_friction: f32 = self.const_air_friction;
		
		if self.gravity {
			// Apply Friction
			self.velocity.x *= air_friction;
			self.velocity.z *= air_friction;
			
			// Attempt to jump...
			if window.get_key(Key::Space) == Action::Press && is_on_ground {
				self.impulse.y += self.const_jump_power;
			}
		} else {
			// Apply Friction
			self.velocity *= air_friction;
		}
	}
}

// This impl-block has to deal with OpenGL shenanigans. Do NOT use for anything but rendering.
impl crate::render::camera::Camera for Freecam {
	fn get_gl_position(&self, interpolation: f32) -> nalgebra_glm::Vec3 {
		self.get_position(interpolation) + self.get_eye_offset(interpolation)
	}
	
	fn get_gl_rotation_matrix(&self, _interpolation: f32) -> nalgebra_glm::Mat4 {
		let pitch = self.rotation.x.to_radians();
		let yaw   = self.rotation.y.to_radians();
		
		let yaw = nalgebra_glm::quat_angle_axis(yaw, &nalgebra_glm::vec3(0.0, 1.0, 0.0));
		let pitch = nalgebra_glm::quat_angle_axis(pitch, &nalgebra_glm::vec3(1.0, 0.0, 0.0));
		
		nalgebra_glm::quat_to_mat4(&nalgebra_glm::quat_cross(&pitch, &yaw))
	}
	
	fn get_gl_projection_matrix(&self, viewport: (i32, i32), _interpolation: f32) -> nalgebra_glm::Mat4 {
		let (width, height) = viewport;
		
		// Apply velocity to the FoV for speedy-effect
		let field_of_view = if self.fov_vel_effect {
			self.field_of_view + self.velocity.magnitude() * 23.42
		} else {
			self.field_of_view
		};
		
		nalgebra_glm::perspective(width as f32 / height as f32, field_of_view, self.min_depth, self.max_depth)
	}
}

impl std::fmt::Display for Freecam {
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
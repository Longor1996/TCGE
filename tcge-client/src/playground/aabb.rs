use cgmath::{Vector3, Zero};
use std::ops::Neg;

#[derive(Copy, Clone, Debug)]
pub struct AxisAlignedBoundingBox {
	pub x_min: f32,
	pub x_max: f32,
	pub y_min: f32,
	pub y_max: f32,
	pub z_min: f32,
	pub z_max: f32,
}

impl AxisAlignedBoundingBox {
	pub fn from_extent(extent: f32) -> Self {
		Self {
			x_min: -extent,
			y_min: -extent,
			z_min: -extent,
			x_max: extent,
			y_max: extent,
			z_max: extent,
		}
	}
	
	pub fn from_position_radius_height(position: Vector3<f32>, radius: f32, height: f32) -> Self {
		Self {
			x_min: position.x - radius,
			y_min: position.y - height,
			z_min: position.z - radius,
			x_max: position.x + radius,
			y_max: position.y + height,
			z_max: position.z + radius,
		}
	}
	
	pub fn from_position_size(position: Vector3<f32>, size: Vector3<f32>) -> Self {
		Self {
			x_min: position.x,
			y_min: position.y,
			z_min: position.z,
			x_max: position.x + size.x,
			y_max: position.y + size.y,
			z_max: position.z + size.z,
		}
	}
}

impl Default for AxisAlignedBoundingBox {
	fn default() -> Self {
		Self {
			x_min: -1.0,
			y_min: -1.0,
			z_min: -1.0,
			x_max: 1.0,
			y_max: 1.0,
			z_max: 1.0,
		}
	}
}

impl AxisAlignedBoundingBox {
	
	pub fn min_vec(&self) -> Vector3<f32> {
		Vector3::new(self.x_min, self.y_min, self.z_min)
	}
	
	pub fn max_vec(&self) -> Vector3<f32> {
		Vector3::new(self.x_max, self.y_max, self.z_max)
	}
	
	pub fn min(&self, idx: usize) -> f32 {
		match idx {
			0 => self.x_min,
			1 => self.y_min,
			2 => self.z_min,
			_ => panic!("Invalid axis id: {}", idx)
		}
	}
	
	pub fn max(&self, idx: usize) -> f32 {
		match idx {
			0 => self.x_max,
			1 => self.y_max,
			2 => self.z_max,
			_ => panic!("Invalid axis id: {}", idx)
		}
	}
	
	pub fn center(&self) -> Vector3<f32> {
		Vector3::new(
			(self.x_min + self.x_max) / 2.0,
			(self.y_min + self.y_max) / 2.0,
			(self.z_min + self.z_max) / 2.0
		)
	}
	
	pub fn dimensions(&self) -> Vector3<f32> {
		Vector3::new(
			(self.x_max - self.x_min),
			(self.y_max - self.y_min),
			(self.z_max - self.z_min)
		)
	}
	
	pub fn extent(&self) -> Vector3<f32> {
		self.dimensions() / 2.0
	}
	
	pub fn contains(&self, p: Vector3<f32>) -> bool {
		   p.x >= self.x_min && p.x <= self.x_max
		&& p.y >= self.y_min && p.y <= self.y_max
		&& p.z >= self.z_min && p.z <= self.z_max
	}
	
	pub fn contains_with_delta(&self, p: Vector3<f32>, d: f32) -> bool {
		   p.x >= self.x_min-d && p.x <= self.x_max+d
		&& p.y >= self.y_min-d && p.y <= self.y_max+d
		&& p.z >= self.z_min-d && p.z <= self.z_max+d
	}
	
	pub fn nearest_corner(&self, p: Vector3<f32>) -> Vector3<f32> {
		fn nearest(v: f32, a: f32, b: f32) -> f32 {
			if (a - v).abs() < (b - v).abs() {a} else {b}
		};
		Vector3::new(
			nearest(p.x, self.x_min, self.x_max),
			nearest(p.y, self.y_min, self.y_max),
			nearest(p.z, self.z_min, self.z_max)
		)
	}
	
	pub fn distance_squared(&self, other: &Self) -> f32 {
		let d = self.center() - other.center();
		d.x*d.x + d.y*d.y + d.z*d.z
	}
	
	pub fn distance(&self, other: &Self) -> f32 {
		let d = self.distance_squared(other);
		if d == 0.0 {0.0} else {d.sqrt()}
	}
}

impl AxisAlignedBoundingBox {
	pub fn intersect(&self, other: &Self) -> bool {
		if self.x_max < other.x_min || self.x_min > other.x_max { return false; }
		if self.y_max < other.y_min || self.y_min > other.y_max { return false; }
		if self.z_max < other.z_min || self.z_min > other.z_max { return false; }
		true
	}
	
	pub fn intersection_x(&self, other: &Self, mut delta: f32) -> f32 {
		if other.y_max > self.y_min && other.y_min < self.y_max {
			if other.z_max > self.z_min && other.z_min < self.z_max {
				let mut d1 = 0.0;
				
				if delta > 0.0 && other.x_max <= self.x_min {
					d1 = self.x_min - other.x_max;
					if d1 < delta {
						delta = d1;
					}
				}
				
				if delta < 0.0 && other.x_min >= self.x_max {
					d1 = self.x_max - other.x_min;
					if d1 > delta {
						delta = d1;
					}
				}
				
				delta
			} else {
				delta
			}
		} else {
			delta
		}
	}
	
	pub fn intersection_y(&self, other: &Self, mut delta: f32) -> f32 {
		if other.x_max > self.x_min && other.x_min < self.x_max {
			if other.z_max > self.z_min && other.z_min < self.z_max {
				let mut d1 = 0.0;
				
				if delta > 0.0 && other.y_max <= self.y_min {
					d1 = self.y_min - other.y_max;
					if d1 < delta {
						delta = d1;
					}
				}
				
				if delta < 0.0 && other.y_min >= self.y_max {
					d1 = self.y_max - other.y_min;
					if d1 > delta {
						delta = d1;
					}
				}
				
				delta
			} else {
				delta
			}
		} else {
			delta
		}
	}
	
	pub fn intersection_z(&self, other: &Self, mut delta: f32) -> f32 {
		if other.x_max > self.x_min && other.x_min < self.x_max {
			if other.y_max > self.y_min && other.y_min < self.y_max {
				let mut d1 = 0.0;
				
				if delta > 0.0 && other.z_max <= self.z_min {
					d1 = self.z_min - other.z_max;
					if d1 < delta {
						delta = d1;
					}
				}
				
				if delta < 0.0 && other.z_min >= self.z_max {
					d1 = self.z_max - other.z_min;
					if d1 > delta {
						delta = d1;
					}
				}
				
				delta
			} else {
				delta
			}
		} else {
			delta
		}
	}
	
}
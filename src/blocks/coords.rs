#[derive(Eq, Clone, Debug)]
pub struct BlockCoord {
	pub x: isize,
	pub y: isize,
	pub z: isize,
}

impl BlockCoord {
	pub fn new(x: isize, y: isize, z: isize) -> Self {
		Self {
			x, y, z
		}
	}
	
	pub fn set(&mut self, x: isize, y: isize, z: isize) {
		self.x = x;
		self.y = y;
		self.z = z;
	}
	
	pub fn as_vec(&self) -> cgmath::Vector3<f32> {
		cgmath::Vector3 {
			x: self.x as f32,
			y: self.y as f32,
			z: self.z as f32
		}
	}
	
	pub fn add(&self, x: isize, y: isize, z: isize) -> Self {
		Self {
			x: self.x + x,
			y: self.y + y,
			z: self.z + z,
		}
	}
	
	pub fn sub(&self, x: isize, y: isize, z: isize) -> Self {
		Self {
			x: self.x - x,
			y: self.y - y,
			z: self.z - z,
		}
	}
	
	pub fn up(&self, distance: isize) -> Self {
		Self {
			x: self.x,
			y: self.y + distance,
			z: self.z,
		}
	}
	
	pub fn down(&self, distance: isize) -> Self {
		Self {
			x: self.x,
			y: self.y - distance,
			z: self.z,
		}
	}
	
	pub fn left(&self, distance: isize) -> Self {
		Self {
			x: self.x - distance,
			y: self.y,
			z: self.z,
		}
	}
	
	pub fn right(&self, distance: isize) -> Self {
		Self {
			x: self.x + distance,
			y: self.y,
			z: self.z,
		}
	}
	
	pub fn forward(&self, distance: isize) -> Self {
		Self {
			x: self.x,
			y: self.y,
			z: self.z - distance,
		}
	}
	
	pub fn backward(&self, distance: isize) -> Self {
		Self {
			x: self.x,
			y: self.y,
			z: self.z + distance,
		}
	}
}

impl PartialEq for BlockCoord {
	fn eq(&self, other: &Self) -> bool {
		self.x == other.x
			&& self.y == other.y
			&& self.z == other.z
	}
}

impl std::fmt::Display for BlockCoord {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "[x: {}, y: {}, z: {}]",
			self.x,
			self.y,
			self.z,
		)
	}
}

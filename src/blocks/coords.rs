#[derive(Eq, Clone, Debug)]
pub struct BlockCoord {
	pub x: isize,
	pub y: isize,
	pub z: isize,
}

impl BlockCoord {
	pub fn new(x: isize, y: isize, z: isize) -> BlockCoord {
		BlockCoord {
			x, y, z
		}
	}
	
	pub fn as_vec(&self) -> cgmath::Vector3<f32> {
		cgmath::Vector3 {
			x: self.x as f32,
			y: self.y as f32,
			z: self.z as f32
		}
	}
}

impl PartialEq for BlockCoord {
	fn eq(&self, other: &BlockCoord) -> bool {
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

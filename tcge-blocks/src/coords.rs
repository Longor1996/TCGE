pub type BlockDim = i32;

#[derive(Eq, Clone, Debug)]
pub struct BlockCoord {
	pub x: BlockDim,
	pub y: BlockDim,
	pub z: BlockDim,
}



impl BlockCoord {
	pub fn new(x: BlockDim, y: BlockDim, z: BlockDim) -> Self {
		Self {
			x, y, z
		}
	}
	
	pub fn set(&mut self, x: BlockDim, y: BlockDim, z: BlockDim) {
		self.x = x;
		self.y = y;
		self.z = z;
	}
}



impl BlockCoord {
	pub fn add(&self, x: BlockDim, y: BlockDim, z: BlockDim) -> Self {
		Self {
			x: self.x + x,
			y: self.y + y,
			z: self.z + z,
		}
	}
	
	pub fn sub(&self, x: BlockDim, y: BlockDim, z: BlockDim) -> Self {
		Self {
			x: self.x - x,
			y: self.y - y,
			z: self.z - z,
		}
	}
	
	pub fn up(&self, distance: BlockDim) -> Self {
		Self {
			x: self.x,
			y: self.y + distance,
			z: self.z,
		}
	}
	
	pub fn down(&self, distance: BlockDim) -> Self {
		Self {
			x: self.x,
			y: self.y - distance,
			z: self.z,
		}
	}
	
	pub fn left(&self, distance: BlockDim) -> Self {
		Self {
			x: self.x - distance,
			y: self.y,
			z: self.z,
		}
	}
	
	pub fn right(&self, distance: BlockDim) -> Self {
		Self {
			x: self.x + distance,
			y: self.y,
			z: self.z,
		}
	}
	
	pub fn forward(&self, distance: BlockDim) -> Self {
		Self {
			x: self.x,
			y: self.y,
			z: self.z - distance,
		}
	}
	
	pub fn backward(&self, distance: BlockDim) -> Self {
		Self {
			x: self.x,
			y: self.y,
			z: self.z + distance,
		}
	}
	
	pub fn negate(&self) -> Self {
		Self {
			x: -self.x,
			y: -self.y,
			z: -self.z,
		}
	}
}



impl Default for BlockCoord {
	/// Returns None.
	#[inline]
	fn default() -> Self {
		Self {
			x: 0,
			y: 0,
			z: 0
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
		write!(fmt, "({}, {}, {})",
			self.x,
			self.y,
			self.z,
		)
	}
}



impl From<(BlockDim, BlockDim, BlockDim)> for BlockCoord {
	fn from(other: (BlockDim, BlockDim, BlockDim)) -> Self {
		Self::new(other.0, other.1, other.2)
	}
}

impl From<(f32, f32, f32)> for BlockCoord {
	fn from(other: (f32, f32, f32)) -> Self {
		Self::new(
			other.0.floor() as BlockDim,
			other.1.floor() as BlockDim,
			other.2.floor() as BlockDim
		)
	}
}



impl Into<(BlockDim, BlockDim, BlockDim)> for BlockCoord {
	fn into(self) -> (BlockDim, BlockDim, BlockDim) {
		(self.x, self.y, self.z)
	}
}

impl Into<(f32, f32, f32)> for BlockCoord {
	fn into(self) -> (f32, f32, f32) {
		(self.x as f32, self.y as f32, self.z as f32)
	}
}

impl Into<(f64, f64, f64)> for BlockCoord {
	fn into(self) -> (f64, f64, f64) {
		(self.x as f64, self.y as f64, self.z as f64)
	}
}

use super::*;

pub type ChunkDim = i32;

#[derive(Eq, Copy, Clone)]
pub struct ChunkCoord {
	pub x: ChunkDim,
	pub y: ChunkDim,
	pub z: ChunkDim,
}

impl ChunkCoord {
	pub fn new_from_chunk(x: ChunkDim, y: ChunkDim, z: ChunkDim) -> ChunkCoord {
		ChunkCoord {
			x, y, z
		}
	}
	
	pub fn new_from_block(pos: &BlockCoord) -> ChunkCoord {
		ChunkCoord {
			x: pos.x >> CHUNK_SIZE_BITS as i32,
			y: pos.y >> CHUNK_SIZE_BITS as i32,
			z: pos.z >> CHUNK_SIZE_BITS as i32,
		}
	}
	
	#[allow(dead_code)]
	pub fn set(&mut self, x: ChunkDim, y: ChunkDim, z: ChunkDim) {
		self.x = x;
		self.y = y;
		self.z = z;
	}
	
	#[allow(dead_code)]
	pub fn add(&self, x: ChunkDim, y: ChunkDim, z: ChunkDim) -> Self {
		Self {
			x: self.x + x,
			y: self.y + y,
			z: self.z + z,
		}
	}
	
	#[allow(dead_code)]
	pub fn sub(&self, x: ChunkDim, y: ChunkDim, z: ChunkDim) -> Self {
		Self {
			x: self.x - x,
			y: self.y - y,
			z: self.z - z,
		}
	}
	
	pub fn contains_block(&self, block: &BlockCoord) -> bool {
		let (cx, cy, cz) = self.to_block_coord_tuple();
		   block.x >= cx && block.x < cx+CHUNK_SIZE_I
		&& block.y >= cy && block.y < cy+CHUNK_SIZE_I
		&& block.z >= cz && block.z < cz+CHUNK_SIZE_I
	}
	
	pub fn to_block_coord_tuple(&self) -> (ChunkDim, ChunkDim, ChunkDim) {
		(self.x * CHUNK_SIZE_I, self.y * CHUNK_SIZE_I, self.z * CHUNK_SIZE_I)
	}
	
	pub fn to_block_coord(&self) -> BlockCoord {
		BlockCoord::new(self.x * CHUNK_SIZE_I, self.y * CHUNK_SIZE_I, self.z * CHUNK_SIZE_I)
	}
}

impl PartialEq for ChunkCoord {
	fn eq(&self, other: &ChunkCoord) -> bool {
		self.x == other.x
			&& self.y == other.y
			&& self.z == other.z
	}
}

impl std::hash::Hash for ChunkCoord {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		(self.x).hash(state);
		(self.z).hash(state);
		(self.y).hash(state);
	}
}

impl std::fmt::Display for ChunkCoord {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "[x: {}, y: {}, z: {}]",
			self.x,
			self.y,
			self.z,
		)
	}
}
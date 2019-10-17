use super::*;

pub type ChunkWithEdge = Box<[[[BlockState; CHUNK_SIZE + 2]; CHUNK_SIZE + 2]; CHUNK_SIZE + 2]>;

pub struct Chunk {
	pub pos: ChunkCoord,
	pub blocks: BlocksRef,
	pub data: Box<[BlockState; CHUNK_VOLUME]>,
	pub last_update: u128
}

impl Chunk {
	
	pub fn new(blocks: &BlocksRef, pos: ChunkCoord, state: BlockState) -> Self {
		let blocks = blocks.clone();
		let data = Box::new([state; CHUNK_VOLUME]);
		
		Self {
			pos,
			blocks,
			data,
			last_update: 0,
		}
	}
	
	#[inline]
	pub fn clamp_chunk_coord(value: BlockDim) -> Option<BlockDim> {
		if value < 0 {
			return None
		}
		
		if value >= CHUNK_SIZE_I {
			return None
		}
		
		Some(value as BlockDim)
	}
	
	#[inline]
	pub fn coord_to_index(x: BlockDim, y: BlockDim, z: BlockDim) -> usize {
		// 'God Tier' indexing function from hell.
		((((y << CHUNK_SIZE_BITS_I) | z) << CHUNK_SIZE_BITS_I) | x) as usize
	}
}

impl Chunk {
	pub fn get_block(&self, x: BlockDim, y: BlockDim, z: BlockDim) -> Option<BlockState> {
		let x = Chunk::clamp_chunk_coord(x)?;
		let y = Chunk::clamp_chunk_coord(y)?;
		let z = Chunk::clamp_chunk_coord(z)?;
		unsafe {
			Some(*self.data.get_unchecked(Self::coord_to_index(x, y, z)))
		}
	}
	
	/// This function returns the block at the given local coordinate, without doing *any* boundary checks.
	///
	/// If the passed-in coordinates are not in the range 0..CHUNK_SIZE, expect invalid data and segmentation faults.
	#[inline]
	pub unsafe fn get_block_unchecked(&self, x: BlockDim, y: BlockDim, z: BlockDim) -> BlockState {
		// This function performs ZERO boundary checks!!!
		*self.data.get_unchecked(Self::coord_to_index(x, y, z))
	}
	
	pub fn set_block(&mut self, x: BlockDim, y: BlockDim, z: BlockDim, state: BlockState) -> Option<()> {
		let x = Chunk::clamp_chunk_coord(x)?;
		let y = Chunk::clamp_chunk_coord(y)?;
		let z = Chunk::clamp_chunk_coord(z)?;
		
		let index = y*CHUNK_SLICE_I + z*CHUNK_SIZE_I + x;
		
		if self.data[index as usize] == state {
			return None
		}
		
		self.data[index as usize] = state;
		self.last_update = current_time_nanos();
		Some(())
	}
}

impl Chunk {
	pub fn fill_with_floor(&mut self, fill: BlockState) {
		for z in 0..CHUNK_SIZE_I {
			for x in 0..CHUNK_SIZE_I {
				self.set_block(x, 0, z, fill);
			}
		}
	}
	
	pub fn fill_with_grid(&mut self, fill: BlockState) {
		const I: BlockDim = CHUNK_SIZE_I - 1;
		for i in 0 .. CHUNK_SIZE_I {
			self.set_block(i,0,0,fill);
			self.set_block(i,I,0,fill);
			self.set_block(i,0,I,fill);
			self.set_block(i,I,I,fill);
			self.set_block(0,i,0,fill);
			self.set_block(I,i,0,fill);
			self.set_block(0,i,I,fill);
			self.set_block(I,i,I,fill);
			self.set_block(0,0,i,fill);
			self.set_block(I,0,i,fill);
			self.set_block(0,I,i,fill);
			self.set_block(I,I,i,fill);
		}
	}
}

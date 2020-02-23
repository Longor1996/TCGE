use super::*;

pub struct ChunkStorage {
	pub blocks: BlocksRef,
	pub chunks: FxHashMap<ChunkCoord, Chunk>,
}

impl ChunkStorage {
	pub fn new(blocks: &BlocksRef) -> Self {
		let mut storage = Self {
			blocks: blocks.clone(),
			chunks: FxHashMap::default()
		};
		
		let range: ChunkDim = 4;
		let height: ChunkDim = 3;
		
		storage.chunks.reserve((
			(height)
				* (range*2 +1)
				* (range*2 +1)
		) as usize);
		
		let air = blocks
			.get_block_by_name_unchecked("air")
			.get_default_state();
		
		// Get the only basic solid...
		let bedrock = blocks
			.get_block_by_name_unchecked("adm")
			.get_default_state();
		
		for y in 0..height {
			for z in -range..range {
				for x in -range..range {
					let mut chunk = Chunk::new(blocks, ChunkCoord::new_from_chunk(x, y, z), air);
					
					if chunk.pos.y == 0 {
						chunk.fill_with_floor(bedrock);
					}
					
					// chunk.fill_with_noise(bedrock, 0.1);
					chunk.fill_with_grid(bedrock);
					storage.chunks.insert(chunk.pos, chunk);
				}
			}
		}
		
		storage
	}
	
	pub fn get_chunk(&self, pos: &ChunkCoord) -> Option<&Chunk> {
		self.chunks.get(pos)
	}
	
	pub fn get_chunk_mut(&mut self, pos: &ChunkCoord) -> Option<&mut Chunk> {
		self.chunks.get_mut(pos)
	}
	
	pub fn get_chunk_with_edges(&self, pos: &ChunkCoord) -> Option<ChunkWithEdge> {
		let chunk = self.get_chunk(pos)?;
		let chunk_pos = chunk.pos.clone();
		let chunk_block_pos = chunk_pos.to_block_coord();
		
		let air = self.blocks.get_block_by_name_unchecked("air").get_default_state();
		
		let mut output = [[[air; CHUNK_SIZE + 2]; CHUNK_SIZE + 2]; CHUNK_SIZE + 2];
		
		for y in 0..CHUNK_SIZE {
			for z in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					output[y+1][z+1][x+1] = unsafe {
						chunk.get_block_unchecked(
							x as BlockDim,
							y as BlockDim,
							z as BlockDim,
						)
					};
				}
			}
		}
		
		// --- Neighbour Faces
		
		// BOTTOM Face...
		if let Some(neighbour) = self.get_chunk(&pos.add(0, -1, 0)) {
			let offset_self= 0;
			let offset_face = CHUNK_SIZE_I-1;
			for z in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					output[offset_self][z+1][x+1] = unsafe {
						neighbour.get_block_unchecked(
							x as BlockDim,
							offset_face,
							z as BlockDim,
						)
					};
				}
			}
		}
		
		// TOP Face...
		if let Some(neighbour) = self.get_chunk(&pos.add(0, 1, 0)) {
			let offset_self= CHUNK_SIZE+1;
			let offset_face = 0;
			for z in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					output[offset_self][z+1][x+1] = unsafe {
						neighbour.get_block_unchecked(
							x as BlockDim,
							offset_face,
							z as BlockDim,
						)
					};
				}
			}
		}
		
		// FRONT Face
		if let Some(neighbour) = self.get_chunk(&pos.add(0, 0, -1)) {
			let offset_self= 0;
			let offset_face = CHUNK_SIZE_I-1;
			for y in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					output[y+1][offset_self][x+1] = unsafe {
						neighbour.get_block_unchecked(
							x as BlockDim,
							y as BlockDim,
							offset_face,
						)
					};
				}
			}
		}
		
		// BACK Face
		if let Some(neighbour) = self.get_chunk(&pos.add(0, 0, 1)) {
			let offset_self= CHUNK_SIZE+1;
			let offset_face = 0;
			for y in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					output[y+1][offset_self][x+1] = unsafe {
						neighbour.get_block_unchecked(
							x as BlockDim,
							y as BlockDim,
							offset_face,
						)
					};
				}
			}
		}
		
		// LEFT Face
		if let Some(neighbour) = self.get_chunk(&pos.add(-1, 0, 0)) {
			let offset_self= 0;
			let offset_face = CHUNK_SIZE_I-1;
			for y in 0..CHUNK_SIZE {
				for z in 0..CHUNK_SIZE {
					output[y+1][z+1][offset_self] = unsafe {
						neighbour.get_block_unchecked(
							offset_face,
							y as BlockDim,
							z as BlockDim,
						)
					};
				}
			}
		}
		
		// RIGHT Face
		if let Some(neighbour) = self.get_chunk(&pos.add(1, 0, 0)) {
			let offset_self= CHUNK_SIZE+1;
			let offset_face = 0;
			for y in 0..CHUNK_SIZE {
				for z in 0..CHUNK_SIZE {
					output[y+1][z+1][offset_self] = unsafe {
						neighbour.get_block_unchecked(
							offset_face,
							y as BlockDim,
							z as BlockDim,
						)
					};
				}
			}
		}
		
		{ // Copy the edges...
			/*  Y  Z  X
				0, 0, i
				I, 0, i
				0, I, i
				I, I, i
				i, 0, 0
				i, 0, I
				i, I, 0
				i, I, I
				0, i, 0
				0, i, I
				I, i, 0
				I, i, I
			*/
			let m = CHUNK_SIZE + 1;
			for i in 0..CHUNK_SIZE+2 {
				let ib = i as BlockDim - 1;
				let mb = CHUNK_SIZE_I;
				output[0][0][i] = self.get_block(&chunk_block_pos.add(ib, -1, -1)).unwrap_or(air);
				output[m][0][i] = self.get_block(&chunk_block_pos.add(ib, mb, -1)).unwrap_or(air);
				output[0][m][i] = self.get_block(&chunk_block_pos.add(ib, -1, mb)).unwrap_or(air);
				output[m][m][i] = self.get_block(&chunk_block_pos.add(ib, mb, mb)).unwrap_or(air);
				output[i][0][0] = self.get_block(&chunk_block_pos.add(-1, ib, -1)).unwrap_or(air);
				output[i][0][m] = self.get_block(&chunk_block_pos.add(mb, ib, -1)).unwrap_or(air);
				output[i][m][0] = self.get_block(&chunk_block_pos.add(-1, ib, mb)).unwrap_or(air);
				output[i][m][m] = self.get_block(&chunk_block_pos.add(mb, ib, mb)).unwrap_or(air);
				output[0][i][0] = self.get_block(&chunk_block_pos.add(-1, -1, ib)).unwrap_or(air);
				output[0][i][m] = self.get_block(&chunk_block_pos.add(mb, -1, ib)).unwrap_or(air);
				output[m][i][0] = self.get_block(&chunk_block_pos.add(-1, mb, ib)).unwrap_or(air);
				output[m][i][m] = self.get_block(&chunk_block_pos.add(mb, mb, ib)).unwrap_or(air);
			}
		}
		
		// Copy the corners...
		{
			let m = CHUNK_SIZE + 1;
			let mb = CHUNK_SIZE_I;
			
			////// Y  Z  X
			output[0][0][0] = self.get_block(&chunk_block_pos.add(-1, -1, -1)).unwrap_or(air);
			output[0][0][m] = self.get_block(&chunk_block_pos.add(mb, -1, -1)).unwrap_or(air);
			output[m][0][0] = self.get_block(&chunk_block_pos.add(-1, mb, -1)).unwrap_or(air);
			output[m][0][m] = self.get_block(&chunk_block_pos.add(mb, mb, -1)).unwrap_or(air);
			output[0][m][0] = self.get_block(&chunk_block_pos.add(-1, -1, mb)).unwrap_or(air);
			output[0][m][m] = self.get_block(&chunk_block_pos.add(mb, -1, mb)).unwrap_or(air);
			output[m][m][0] = self.get_block(&chunk_block_pos.add(-1, mb, mb)).unwrap_or(air);
			output[m][m][m] = self.get_block(&chunk_block_pos.add(mb, mb, mb)).unwrap_or(air);
		}
		
		Some(Box::new(output))
	}
}

impl ChunkStorage {
	pub fn get_block(&self, pos: &BlockCoord) -> Option<BlockState> {
		let chunk_pos = ChunkCoord::new_from_block(pos);
		
		if let Some(chunk) = self.get_chunk(&chunk_pos) {
			let cx = pos.x & CHUNK_SIZE_MASK_I;
			let cy = pos.y & CHUNK_SIZE_MASK_I;
			let cz = pos.z & CHUNK_SIZE_MASK_I;
			match chunk.get_block(cx, cy, cz) {
				Some(x) => return Some(x),
				None => ()
			}
		}
		
		None
	}
	
	pub fn set_block(&mut self, pos: &BlockCoord, state: BlockState) -> bool {
		let chunk_pos = ChunkCoord::new_from_block(pos);
		let cx = pos.x & CHUNK_SIZE_MASK_I;
		let cy = pos.y & CHUNK_SIZE_MASK_I;
		let cz = pos.z & CHUNK_SIZE_MASK_I;
		
		let success = if let Some(chunk) = self.get_chunk_mut(&chunk_pos) {
			match chunk.set_block(cx, cy, cz, state) {
				Some(_) => true,
				None    => false
			}
		} else {
			false
		};
		
		if success {
			let now = current_time_nanos();
			if cx == 0 {
				self.get_chunk_mut(&chunk_pos.add(-1,0,0))
					.map(|c| {c.last_update = now});
			}
			if cy == 0 {
				self.get_chunk_mut(&chunk_pos.add(0,-1,0))
					.map(|c| {c.last_update = now});
			}
			if cz == 0 {
				self.get_chunk_mut(&chunk_pos.add(0,0,-1))
					.map(|c| {c.last_update = now});
			}
			if cx == CHUNK_SIZE_I-1 {
				self.get_chunk_mut(&chunk_pos.add(1,0,0))
					.map(|c| {c.last_update = now});
			}
			if cy == CHUNK_SIZE_I-1 {
				self.get_chunk_mut(&chunk_pos.add(0,1,0))
					.map(|c| {c.last_update = now});
			}
			if cz == CHUNK_SIZE_I-1 {
				self.get_chunk_mut(&chunk_pos.add(0,0,1))
					.map(|c| {c.last_update = now});
			}
		}
		
		success
	}
	
	pub fn raycast(&mut self, raycast: &mut BlockRaycast) -> BlockRaycastResponse {
		loop {
			let last = raycast.previous();
			
			let current = match raycast.step() {
				Some(pos) => pos,
				None => break
			};
			
			let last_pos = BlockCoord::new(last.0, last.1, last.2);
			let current_pos = BlockCoord::new(current.0, current.1, current.2);
			
			let air = self.blocks
				.get_block_by_name_unchecked("air")
				.get_default_state();
			
			match self.get_block(&current_pos) {
				Some(block) => {
					if block != air {
						return Some((last_pos, current_pos, block))
					}
				}
				_ => ()
			}
		}
		
		None
	}
	
	pub fn get_approximate_volume(&self) -> u64 {
		(self.chunks.len() as u64) * (CHUNK_VOLUME as u64)
	}
}

pub type BlockRaycastResponse = Option<(BlockCoord, BlockCoord, BlockState)>;

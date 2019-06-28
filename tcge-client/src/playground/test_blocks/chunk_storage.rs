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
		
		// Get the only basic solid...
		let bedrock = blocks
			.get_block_by_name_unchecked("adm")
			.get_default_state();
		
		for y in 0..height {
			for z in -range..range {
				for x in -range..range {
					let mut chunk = Chunk::new(blocks, ChunkCoord::new_from_chunk(x, y, z));
					
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
	
	pub fn get_chunks_around(&self, pos: &ChunkCoord) -> [Option<&Chunk>;27] {
		let positions = [
			// bottom layer
			pos.add(-1,-1,-1),
			pos.add( 0,-1,-1),
			pos.add( 1,-1,-1),
			pos.add(-1,-1, 0),
			pos.add( 0,-1, 0), // bottom chunk
			pos.add( 1,-1, 0),
			pos.add(-1,-1, 1),
			pos.add( 0,-1, 1),
			pos.add( 1,-1, 1),
			
			// middle layer
			pos.add(-1, 0,-1),
			pos.add( 0, 0,-1),
			pos.add( 1, 0,-1),
			pos.add(-1, 0, 0),
			pos.add( 0, 0, 0), // central chunk
			pos.add( 1, 0, 0),
			pos.add(-1, 0, 1),
			pos.add( 0, 0, 1),
			pos.add( 1, 0, 1),
			
			// top layer
			pos.add(-1, 1,-1),
			pos.add( 0, 1,-1),
			pos.add( 1, 1,-1),
			pos.add(-1, 1, 0),
			pos.add( 0, 1, 0), // top chunk
			pos.add( 1, 1, 0),
			pos.add(-1, 1, 1),
			pos.add( 0, 1, 1),
			pos.add( 1, 1, 1),
		];
		
		let mut chunks: [Option<&Chunk>;27] = [None; 27];
		
		for (index, coord) in positions.iter().enumerate() {
			chunks[index] = self.get_chunk(coord);
		}
		
		chunks
	}
}

impl ChunkStorage {
	pub fn get_block(&self, pos: &BlockCoord) -> Option<BlockState> {
		let cpos = ChunkCoord::new_from_block(pos);
		
		if let Some(chunk) = self.get_chunk(&cpos) {
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
		let cpos = ChunkCoord::new_from_block(pos);
		let csm = CHUNK_SIZE_MASK_I;
		
		let success = if let Some(chunk) = self.get_chunk_mut(&cpos) {
			let cx = pos.x & csm;
			let cy = pos.y & csm;
			let cz = pos.z & csm;
			
			match chunk.set_block(cx, cy, cz, state) {
				Some(_) => true,
				None    => false
			}
		} else {
			false
		};
		
		if success {
			let now = current_time_nanos();
			self.get_chunk_mut(&cpos.add(-1,0,0)).map(|c| {c.last_update = now});
			self.get_chunk_mut(&cpos.add(1,0,0)).map(|c| {c.last_update = now});
			self.get_chunk_mut(&cpos.add(0,-1,0)).map(|c| {c.last_update = now});
			self.get_chunk_mut(&cpos.add(0,1,0)).map(|c| {c.last_update = now});
			self.get_chunk_mut(&cpos.add(0,0,-1)).map(|c| {c.last_update = now});
			self.get_chunk_mut(&cpos.add(0,0,1)).map(|c| {c.last_update = now});
		}
		
		success
	}
	
	pub fn raycast(&mut self, raycast: &mut BlockRaycast) -> Option<(BlockCoord, BlockCoord, BlockState)> {
		loop {
			let (lx, ly, lz) = raycast.previous();
			
			let (cx, cy, cz) = match raycast.step() {
				Some(pos) => pos,
				None => break
			};
			
			let last_pos = BlockCoord::new(lx, ly, lz);
			let pos = BlockCoord::new(cx, cy, cz);
			
			let air = self.blocks
				.get_block_by_name_unchecked("air")
				.get_default_state();
			
			match self.get_block(&pos) {
				Some(block) => {
					if block != air {
						return Some((last_pos, pos, block))
					}
				}
				_ => ()
			}
		}
		
		return None;
	}
	
	pub fn get_approximate_volume(&self) -> u64 {
		(self.chunks.len() as u64) * (CHUNK_VOLUME as u64)
	}
}

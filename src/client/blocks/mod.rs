use crate::blocks as blockdef;
use crate::blocks::BlockState;
use crate::blocks::BlockCoord;
use crate::util::current_time_nanos;

const CHUNK_SIZE: usize = 16;
const CHUNK_SIZE_SHIFT: usize = 4;
const CHUNK_SIZE_MASK: usize = 0b1111;

const CHUNK_SLICE: usize = CHUNK_SIZE*CHUNK_SIZE;
const CHUNK_VOLUME: usize = CHUNK_SLICE*CHUNK_SIZE;

pub mod raycast;
pub use self::raycast::BlockRaycast;

pub mod render;
pub mod mesher;
pub use self::render::ChunkRenderManager;

#[derive(Eq, Copy, Clone)]
pub struct ChunkCoord {
	pub x: isize,
	pub y: isize,
	pub z: isize,
}

impl ChunkCoord {
	pub fn new_from_chunk(x: isize, y: isize, z: isize) -> ChunkCoord {
		ChunkCoord {
			x, y, z
		}
	}
	
	pub fn new_from_block(pos: &BlockCoord) -> ChunkCoord {
		ChunkCoord {
			x: pos.x >> CHUNK_SIZE_SHIFT,
			y: pos.y >> CHUNK_SIZE_SHIFT,
			z: pos.z >> CHUNK_SIZE_SHIFT,
		}
	}
	
	pub fn set(&mut self, x: isize, y: isize, z: isize) {
		self.x = x;
		self.y = y;
		self.z = z;
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
	
	pub fn as_vec(&self) -> cgmath::Vector3<f32> {
		cgmath::Vector3 {
			x: self.x as f32,
			y: self.y as f32,
			z: self.z as f32
		}
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

pub struct Chunk {
	pub pos: ChunkCoord,
	pub blockdef: blockdef::UniverseRef,
	pub blocks: Box<[BlockState; CHUNK_VOLUME]>,
	pub last_update: u128
}

impl Chunk {
	
	pub fn new(blockdef: blockdef::UniverseRef, x: isize, y: isize, z: isize) -> Chunk {
		let air = blockdef
			.get_block_by_name_unchecked("air")
			.get_default_state();
		
		Chunk {
			pos: ChunkCoord {x,y,z},
			blockdef: blockdef.clone(),
			blocks: Box::new([air; CHUNK_VOLUME]),
			last_update: current_time_nanos()
		}
	}
	
	pub fn clamp_chunk_coord(value: isize) -> Option<usize> {
		if value < 0 {
			return None
		}
		
		if value >= CHUNK_SIZE as isize {
			return None
		}
		
		return Some(value as usize)
	}
	
	pub fn fill_with_floor(&mut self, fill: BlockState) {
		for z in 0..CHUNK_SIZE as isize {
			for x in 0..CHUNK_SIZE as isize {
				self.set_block(x, 0, z, fill);
			}
		}
	}
	
	pub fn fill_with_grid(&mut self, fill: BlockState) {
		const I: isize = (CHUNK_SIZE - 1) as isize;
		for i in 0..=I {
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
	
	pub fn fill_with_noise(&mut self, fill: BlockState, chance: f64) {
		extern crate rand;
		use rand::prelude::*;
		let mut rng = thread_rng();
		
		for i in self.blocks.iter_mut() {
			if rng.gen_bool(chance) {
				*i = fill
			}
		}
	}
	
	pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<BlockState> {
		let x = Chunk::clamp_chunk_coord(x)?;
		let y = Chunk::clamp_chunk_coord(y)?;
		let z = Chunk::clamp_chunk_coord(z)?;
		
		let index = y*CHUNK_SLICE + z*CHUNK_SIZE + x;
		unsafe {
			Some(*self.blocks.get_unchecked(index))
		}
	}
	
	pub fn set_block(&mut self, x: isize, y: isize, z: isize, state: BlockState) -> Option<()> {
		let x = Chunk::clamp_chunk_coord(x)?;
		let y = Chunk::clamp_chunk_coord(y)?;
		let z = Chunk::clamp_chunk_coord(z)?;
		
		let index = y*CHUNK_SLICE + z*CHUNK_SIZE + x;
		
		if self.blocks[index] == state {
			return None
		}
		
		self.blocks[index] = state;
		self.last_update = current_time_nanos();
		Some(())
	}
	
}

pub struct ChunkStorage {
	blockdef: blockdef::UniverseRef,
	chunks: Vec<Chunk>,
}

impl ChunkStorage {
	pub fn new(
		blockdef: blockdef::UniverseRef,
		config: &toml::value::Table
	) -> ChunkStorage {
		let mut storage = ChunkStorage {
			blockdef: blockdef.clone(),
			chunks: Vec::default()
		};
		
		let mut range = 4;
		let mut height = 3;
		
		if let Some(rv) = config.get("range") {
			if let Some(r) = rv.as_integer() {
				range = r as isize;
			}
		}
		
		if let Some(hv) = config.get("height") {
			if let Some(h) = hv.as_integer() {
				height = h as isize;
			}
		}
		
		// Reserve some memory.
		storage.chunks.reserve((
			(height)
			* (range*2 +1)
			* (range*2 +1)
		) as usize);
		
		// Get the only basic solid...
		let bedrock = blockdef
			.get_block_by_name_unchecked("bedrock")
			.get_default_state();
		
		extern crate fuss;
		use fuss::Simplex;
		let sn = Simplex::new();
		
		for y in 0..height {
			for z in -range..range {
				for x in -range..range {
					let mut chunk = Chunk::new(blockdef.clone(), x, y, z);
					
					if chunk.pos.y == 0 {
						chunk.fill_with_floor(bedrock);
						
						// Determine chunk coordinates within noise-field.
						let bcx = chunk.pos.x as f32 * CHUNK_SIZE as f32;
						let bcz = chunk.pos.z as f32 * CHUNK_SIZE as f32;
						
						// Given the noise, fill in the blocks.
						for bz in 0..CHUNK_SIZE as isize {
							for bx in 0..CHUNK_SIZE as isize {
								
								let block_x = (bcx + bx as f32) / CHUNK_SIZE as f32 / 5.0;
								let block_z = (bcz + bz as f32) / CHUNK_SIZE as f32 / 5.0;
								
								let noise = sn.noise_2d(block_x, block_z);
								let noise = (noise + 1.0) * 0.5;
								let noise = noise * 15.0;
								let n = noise as isize;
								
								if n >= 0 {
									// Fill in block-column
									for by in 0..n {
										chunk.set_block(bx, by, bz, bedrock);
									}
								}
							}
						}
					}
					
					// chunk.fill_with_noise(bedrock, 0.1);
					// chunk.fill_with_grid(bedrock);
					
					// TODO: Add some simple worldgen right here.
					
					storage.chunks.push(chunk);
				}
			}
		}
		
		storage
	}
	
	pub fn get_chunk(&self, pos: &ChunkCoord) -> Option<&Chunk> {
		for chunk in self.chunks.iter() {
			if chunk.pos == *pos {
				return Some(chunk)
			}
		}
		
		return None;
	}
	
	pub fn get_chunk_mut(&mut self, pos: &ChunkCoord) -> Option<&mut Chunk> {
		for chunk in self.chunks.iter_mut() {
			if chunk.pos == *pos {
				return Some(chunk)
			}
		}
		
		return None;
	}
	
	pub fn get_chunks_around(&self, pos: &ChunkCoord) -> [Option<&Chunk>;27] {
		let positions = [
			// bottom layer
			pos.add(-1,-1,-1),
			pos.add( 0,-1,-1),
			pos.add( 1,-1,-1),
			pos.add(-1,-1, 0),
			pos.add( 0,-1, 0),
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
			pos.add( 0, 1, 0),
			pos.add( 1, 1, 0),
			pos.add(-1, 1, 1),
			pos.add( 0, 1, 1),
			pos.add( 1, 1, 1),
		];
		
		let mut chunks: [Option<&Chunk>;27] = [None; 27];
		
		for chunk in self.chunks.iter() {
			let chunk_pos = chunk.pos;
			for (index, position) in positions.iter().enumerate() {
				if chunk_pos == *position {
					chunks[index] = Some(chunk);
				}
			}
		}
		
		chunks
	}
	
	pub fn get_block(&self, pos: &BlockCoord) -> Option<BlockState> {
		let cpos = ChunkCoord::new_from_block(pos);
		let csm = CHUNK_SIZE_MASK as isize;
		
		if let Some(chunk) = self.get_chunk(&cpos) {
			let cx = pos.x & csm;
			let cy = pos.y & csm;
			let cz = pos.z & csm;
			match chunk.get_block(cx, cy, cz) {
				Some(x) => return Some(x),
				None => ()
			}
		}
		
		return None;
	}
	
	pub fn set_block(&mut self, pos: &BlockCoord, state: BlockState) -> bool {
		let cpos = ChunkCoord::new_from_block(pos);
		let csm = CHUNK_SIZE_MASK as isize;
		
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
		
		return success;
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
			
			let air = self.blockdef
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
	
	pub fn raycast_fill(&mut self, raycast: &mut BlockRaycast, state: BlockState) {
		while let Some((x,y,z)) = raycast.step() {
			let pos = BlockCoord::new(x, y, z);
			self.set_block(&pos, state);
		}
	}
	
	pub fn get_approximate_volume(&self) -> u64 {
		(self.chunks.len() as u64) * (CHUNK_VOLUME as u64)
	}
}

use super::super::blocks as blockdef;
use super::super::blocks::BlockState;
use super::super::blocks::BlockCoord;
use super::super::util::current_time_nanos;

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
	pub blocks: [BlockState; CHUNK_VOLUME],
	pub last_update: u128
}

impl Chunk {
	
	pub fn new(blockdef: blockdef::UniverseRef, x: isize, y: isize, z: isize) -> Chunk {
		let air = blockdef
			.get_block_by_name_unchecked("air")
			.get_default_state();
		
		let mut new = Chunk {
			pos: ChunkCoord {x,y,z},
			blockdef: blockdef.clone(),
			blocks: [air; CHUNK_VOLUME],
			last_update: current_time_nanos()
		};
		
		let bedrock = blockdef
			.get_block_by_name_unchecked("bedrock")
			.get_default_state();
		
		// new.fill_with_noise(BLOCK_ADM, 0.1);
		new.fill_with_grid(bedrock);
		
		new
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
		
		for y in 0..height {
			for z in -range..range {
				for x in -range..range {
					let chunk = Chunk::new(blockdef.clone(), x, y, z);
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
		
		if let Some(chunk) = self.get_chunk_mut(&cpos) {
			let cx = pos.x & csm;
			let cy = pos.y & csm;
			let cz = pos.z & csm;
			
			chunk.set_block(cx, cy, cz, state);
			return true;
		}
		
		return false;
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
}

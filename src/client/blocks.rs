use rustc_hash::FxHashMap;

use super::super::resources;
use super::render;

use super::super::blocks as blockdef;
use super::super::blocks::BlockState;
use super::super::blocks::BlockCoord;
use super::super::util::current_time_nanos;

const CHUNK_SIZE: usize = 16;
const CHUNK_SIZE_SHIFT: usize = 4;
const CHUNK_SIZE_MASK: usize = 0b1111;

const CHUNK_SLICE: usize = CHUNK_SIZE*CHUNK_SIZE;
const CHUNK_VOLUME: usize = CHUNK_SLICE*CHUNK_SIZE;

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
	
	pub fn get_block(&self, pos: &BlockCoord) -> Option<BlockState> {
		let cpos = ChunkCoord::new_from_block(pos);
		let csm = CHUNK_SIZE_MASK as isize;
		
		for chunk in self.chunks.iter() {
			if chunk.pos == cpos {
				let cx = pos.x & csm;
				let cy = pos.y & csm;
				let cz = pos.z & csm;
				match chunk.get_block(cx, cy, cz) {
					Some(x) => return Some(x),
					None => ()
				}
			}
		}
		
		return None;
	}
	
	pub fn set_block(&mut self, pos: &BlockCoord, state: BlockState) -> bool {
		let cpos = ChunkCoord::new_from_block(pos);
		let csm = CHUNK_SIZE_MASK as isize;
		
		for chunk in self.chunks.iter_mut() {
			if chunk.pos == cpos {
				let cx = pos.x & csm;
				let cy = pos.y & csm;
				let cz = pos.z & csm;
				
				chunk.set_block(cx, cy, cz, state);
				return true;
			}
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

pub struct BlockRaycast {
	gx: f32,
	gy: f32,
	gz: f32,
	lx: f32,
	ly: f32,
	lz: f32,
	gx1idx: f32,
	gy1idx: f32,
	gz1idx: f32,
	errx: f32,
	erry: f32,
	errz: f32,
	sx: f32,
	sy: f32,
	sz: f32,
	derrx: f32,
	derry: f32,
	derrz: f32,
	done: bool,
	visited: usize,
}

impl BlockRaycast {
	
	pub fn new_from_src_dir_len(src: cgmath::Vector3<f32>, dir: cgmath::Vector3<f32>, len: f32) -> BlockRaycast {
		let dst = src + (dir * len);
		BlockRaycast::new_from_src_dst(src, dst)
	}
	
	pub fn new_from_src_dst(src: cgmath::Vector3<f32>, dst: cgmath::Vector3<f32>) -> BlockRaycast {
		let gx0idx = src.x.floor();
		let gy0idx = src.y.floor();
		let gz0idx = src.z.floor();
		
		let gx1idx = dst.x.floor();
		let gy1idx = dst.y.floor();
		let gz1idx = dst.z.floor();
		
		let sx = BlockRaycast::psign(gx0idx, gx1idx);
		let sy = BlockRaycast::psign(gy0idx, gy1idx);
		let sz = BlockRaycast::psign(gz0idx, gz1idx);
		
		// Planes for each axis that we will next cross
		let gxp = gx0idx + (if gx1idx > gx0idx { 1.0 } else { 0.0 });
		let gyp = gy0idx + (if gy1idx > gy0idx { 1.0 } else { 0.0 });
		let gzp = gz0idx + (if gz1idx > gz0idx { 1.0 } else { 0.0 });
		
		// Only used for multiplying up the error margins
		let vx = if dst.x == src.x { 1.0 } else { dst.x - src.x};
		let vy = if dst.y == src.y { 1.0 } else { dst.y - src.y};
		let vz = if dst.z == src.z { 1.0 } else { dst.z - src.z};
		
		// Error is normalized to vx * vy * vz so we only have to multiply up
		let vxvy = vx * vy;
		let vxvz = vx * vz;
		let vyvz = vy * vz;
		
		// Error from the next plane accumulators, scaled up by vx*vy*vz
		//   gx0 + vx * rx === gxp
		//   vx * rx === gxp - gx0
		//   rx === (gxp - gx0) / vx
		let errx = (gxp - src.x) * vyvz;
		let erry = (gyp - src.y) * vxvz;
		let errz = (gzp - src.z) * vxvy;
		
		let derrx = sx * vyvz;
		let derry = sy * vxvz;
		let derrz = sz * vxvy;
		
		BlockRaycast {
			done: false,
			visited: 0,
			
			gx: gx0idx,
			gy: gy0idx,
			gz: gz0idx,
			lx: gx0idx,
			ly: gy0idx,
			lz: gz0idx,
			gx1idx, gy1idx, gz1idx,
			errx, erry, errz,
			sx, sy, sz,
			derrx, derry, derrz
		}
	}
	
	pub fn current(&self) -> (isize, isize, isize) {
		(
			self.gx as isize,
			self.gy as isize,
			self.gz as isize,
		)
	}
	
	pub fn previous(&self) -> (isize, isize, isize) {
		(
			self.lx as isize,
			self.ly as isize,
			self.lz as isize,
		)
	}
	
	pub fn step(&mut self) -> Option<(isize, isize, isize)> {
		if self.done {
			return None
		}
		
		let ret = (
			self.gx as isize,
			self.gy as isize,
			self.gz as isize,
		);
		
		if self.gx == self.gx1idx && self.gy == self.gy1idx && self.gz == self.gz1idx {
			self.done = true;
		}
		
		self.step_compute();
		self.visited += 1;
		return Some(ret)
	}
	
	fn step_compute(&mut self) {
		self.lx = self.gx;
		self.ly = self.gy;
		self.lz = self.gz;
		
		let xr = self.errx.abs();
		let yr = self.erry.abs();
		let zr = self.errz.abs();
		
		if (self.sx != 0.0) && (self.sy == 0.0 || xr < yr) && (self.sz == 0.0 || xr < zr) {
			self.gx += self.sx;
			self.errx += self.derrx;
		}
		else if (self.sy != 0.0) && (self.sz == 0.0 || yr < zr) {
			self.gy += self.sy;
			self.erry += self.derry;
		}
		else if self.sz != 0.0 {
			self.gz += self.sz;
			self.errz += self.derrz;
		}
	}
	
	fn psign(a: f32, b: f32) -> f32 {
		if b > a {
			1.0
		} else if b < a {
			-1.0
		} else {
			0.0
		}
	}
	
}

pub struct ShaderBlocks {
	pub shader: render::utility::Program,
	pub texatlas: render::utility::Texture,
	pub uniform_matrix: i32,
	pub uniform_atlas: i32,
}

impl ShaderBlocks {
	pub fn new(res: &resources::Resources) -> Result<ShaderBlocks, render::utility::Error> {
		debug!("Loading blocks texture...");
		let texatlas = render::utility::Texture::from_res(&res, "textures/atlas.png", &||{
			unsafe {
				// wrapping
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
				// sampling
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as i32);
				gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
				
				// Attempt to enable anisotropic filtering...
				let mut aniso: f32 = 0.0;
				gl::GetFloatv(0x84FF, &mut aniso);
				if aniso != 0.0 {
					gl::TexParameterf(gl::TEXTURE_2D, 0x84FE, aniso);
				}
			}
		})?;
		
		debug!("Loading blocks shader...");
		let shader = render::utility::Program::from_res(&res, "shaders/blocks")?;
		
		let uniform_matrix = shader.uniform_location("transform");
		let uniform_atlas = shader.uniform_location("atlas");
		
		Ok(ShaderBlocks {shader, texatlas,
			uniform_matrix,
			uniform_atlas,
		})
	}
}

pub struct ChunkRenderManager {
	#[allow(dead_code)] // Not needed... yet.
	blockdef: blockdef::UniverseRef,
	
	chunks: FxHashMap<ChunkCoord, (u128, ChunkMeshState)>,
	material: ShaderBlocks,
	mesher: ChunkMesher,
}

impl ChunkRenderManager {
	pub fn new(res: &resources::Resources, blockdef: blockdef::UniverseRef) -> Result<ChunkRenderManager, render::utility::Error> {
		let material = ShaderBlocks::new(res)?;
		
		Ok(ChunkRenderManager {
			blockdef: blockdef.clone(),
			chunks: FxHashMap::default(),
			material,
			mesher: ChunkMesher::new(blockdef.clone()),
		})
	}
	
	pub fn render(&mut self, scene: &super::scene::Scene, transform: cgmath::Matrix4<f32>) {
		render::utility::gl_push_debug("chunks");
		
		self.material.shader.set_used();
		self.material.shader.uniform_matrix4(self.material.uniform_matrix, transform);
		self.material.shader.uniform_sampler(self.material.uniform_atlas, 0);
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.material.texatlas.id);
		}
		
		let mut max_uploads_per_frame: usize = 1;
		for chunk in scene.chunks.chunks.iter() {
			let cpos = &chunk.pos;
			
			if self.chunks.contains_key(cpos) {
				let (time, mesh) = self.chunks.get_mut(cpos).unwrap();
				
				if chunk.last_update > *time {
					*time = chunk.last_update;
					*mesh = self.mesher.mesh(&chunk);
				}
				
				if let ChunkMeshState::Meshed(mesh) = mesh{
					mesh.draw();
				}
				
			} else {
				if max_uploads_per_frame > 0 {
					max_uploads_per_frame -= 1;
					let mesh = self.mesher.mesh(&chunk);
					self.chunks.insert(cpos.clone(), (current_time_nanos(), mesh));
				}
			}
		}
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
		
		render::utility::gl_pop_debug();
	}
	
}

struct ChunkMesher {
	blockdef: blockdef::UniverseRef,
}

impl ChunkMesher {
	
	fn new(blockdef: blockdef::UniverseRef) -> ChunkMesher {
		ChunkMesher {
			blockdef
		}
	}
	
	fn mesh(&mut self, chunk: &Chunk) -> ChunkMeshState {
		let mut vertices: Vec<ChunkMeshVertex> = vec![];
		
		let cpos = chunk.pos;
		
		const N: f32 = 0.0;
		const S: f32 = 1.0;
		
		let air = self.blockdef
			.get_block_by_name_unchecked("air")
			.get_default_state();
		
		for y in 0..CHUNK_SIZE {
			for z in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					let x = x as isize;
					let y = y as isize;
					let z = z as isize;
					let block = chunk.get_block(x, y, z).unwrap_or(air);
					
					if block == air {
						continue;
					}
					
					let cbp = vertices.len();
					
					// This line is the dumbest thing in the whole project...
					let uv = BlockUv::new_from_pos(block.id.get_raw_id() as u8 - 1, 0);
					// TODO: Implement the static block-bakery.
					
					if chunk.get_block(x,y+1,z).unwrap_or(air) == air {
						Self::quad_to_tris(&[ // top
							(N, S, S, uv.umin, uv.vmin).into(),
							(S, S, S, uv.umax, uv.vmin).into(),
							(S, S, N, uv.umax, uv.vmax).into(),
							(N, S, N, uv.umin, uv.vmax).into(),
						], &mut vertices);
					}
					
					if chunk.get_block(x,y-1,z).unwrap_or(air) == air {
						Self::quad_to_tris(&[ // bottom
							(N, N, N, uv.umin, uv.vmin).into(),
							(S, N, N, uv.umax, uv.vmin).into(),
							(S, N, S, uv.umax, uv.vmax).into(),
							(N, N, S, uv.umin, uv.vmax).into(),
						], &mut vertices);
					}
					
					if chunk.get_block(x,y,z-1).unwrap_or(air) == air {
						Self::quad_to_tris(&[ // front
							(N, S, N, uv.umin, uv.vmin).into(), // a
							(S, S, N, uv.umax, uv.vmin).into(), // b
							(S, N, N, uv.umax, uv.vmax).into(), // c
							(N, N, N, uv.umin, uv.vmax).into(), // d
						], &mut vertices);
					}
					
					if chunk.get_block(x,y,z+1).unwrap_or(air) == air {
						Self::quad_to_tris(&[ // back
							(N, N, S, uv.umin, uv.vmin).into(), // d
							(S, N, S, uv.umax, uv.vmin).into(), // c
							(S, S, S, uv.umax, uv.vmax).into(), // b
							(N, S, S, uv.umin, uv.vmax).into(), // a
						], &mut vertices);
					}
					
					if chunk.get_block(x-1,y,z).unwrap_or(air) == air {
						Self::quad_to_tris(&[ // left
							(N, S, S, uv.umin, uv.vmin).into(), // a
							(N, S, N, uv.umax, uv.vmin).into(), // b
							(N, N, N, uv.umax, uv.vmax).into(), // c
							(N, N, S, uv.umin, uv.vmax).into(), // d
						], &mut vertices);
					}
					
					if chunk.get_block(x+1,y,z).unwrap_or(air) == air {
						Self::quad_to_tris(&[ // right
							(S, N, S, uv.umin, uv.vmin).into(), // d
							(S, N, N, uv.umax, uv.vmin).into(), // c
							(S, S, N, uv.umax, uv.vmax).into(), // b
							(S, S, S, uv.umin, uv.vmax).into(), // a
						], &mut vertices);
					}
					
					for vertex in &mut vertices[cbp..] {
						vertex.x += (x + cpos.x*CHUNK_SIZE as isize) as f32;
						vertex.y += (y + cpos.y*CHUNK_SIZE as isize) as f32;
						vertex.z += (z + cpos.z*CHUNK_SIZE as isize) as f32;
					}
				}
			}
		}
		
		return Self::upload(chunk, &vertices);
	}
	
	fn upload(chunk: &Chunk, mesh_data: &Vec<ChunkMeshVertex>) -> ChunkMeshState {
		// Don't upload empty meshes.
		if mesh_data.len() == 0 {
			return ChunkMeshState::Empty
		}
		
		let vertex_count = mesh_data.len();
		
		let mut vbo: gl::types::GLuint = 0;
		unsafe {
			gl::GenBuffers(1, &mut vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(vertex_count * std::mem::size_of::<ChunkMeshVertex>()) as gl::types::GLsizeiptr,
				mesh_data.as_ptr() as *const gl::types::GLvoid,
				gl::STATIC_DRAW
			);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		
		let mut vao: gl::types::GLuint = 0;
		unsafe {
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
			
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0, // attribute location
				3, // sub-element count
				gl::FLOAT, // sub-element type
				gl::FALSE, // sub-element normalization
				(5 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
				(0 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
			);
			
			gl::EnableVertexAttribArray(1);
			gl::VertexAttribPointer(
				1, // attribute location
				2, // sub-element count
				gl::FLOAT, // sub-element type
				gl::FALSE, // sub-element normalization
				(5 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
				(3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
			);
			
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::BindVertexArray(0);
		}
		
		let label = format!("Chunk({}, {}, {})", chunk.pos.x, chunk.pos.y, chunk.pos.z);
		
		render::utility::gl_label_object(
			gl::VERTEX_ARRAY, vao,
			&format!("{} Descriptor", label)
		);
		
		render::utility::gl_label_object(
			gl::BUFFER, vbo,
			&format!("{} Geometry", label)
		);
		
		return ChunkMeshState::Meshed(ChunkMesh {
			descriptor: vao,
			vertex_buf: vbo,
			count: vertex_count as i32
		})
	}
	
	fn quad_to_tris(src: &[ChunkMeshVertex; 4], dst: &mut Vec<ChunkMeshVertex>) {
		dst.reserve(6);
		dst.push(src[0]);
		dst.push(src[1]);
		dst.push(src[3]);
		dst.push(src[1]);
		dst.push(src[2]);
		dst.push(src[3]);
	}
	
}

/// The graphical state of a chunk.
enum ChunkMeshState {
	/// Chunk is meshed but empty.
	Empty,
	
	/// Chunk is meshed and full.
	Meshed(ChunkMesh),
}

/// The graphical representation of a chunk.
/// Really just a bag of OpenGL Object-Handles.
struct ChunkMesh {
	descriptor: gl::types::GLuint,
	vertex_buf: gl::types::GLuint,
	count: i32,
}

impl ChunkMesh {
	pub fn draw(&self) {
		unsafe {
			gl::BindVertexArray(self.descriptor);
			gl::DrawArrays(gl::TRIANGLES, 0, self.count);
		}
	}
}

impl Drop for ChunkMesh {
	fn drop(&mut self) {
		unsafe {
			let tmp = [self.vertex_buf];
			gl::DeleteBuffers(1, tmp.as_ptr());
			
			let tmp = [self.descriptor];
			gl::DeleteVertexArrays(1, tmp.as_ptr());
		}
	}
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct ChunkMeshVertex {
	// Geometry
	pub x: f32,
	pub y: f32,
	pub z: f32,
	
	// Texture
	pub u: f32,
	pub v: f32,
}

impl ChunkMeshVertex {
	pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
		Self {
			x, y, z, u, v
		}
	}
}

impl From<(f32, f32, f32, f32, f32)> for ChunkMeshVertex {
	fn from(other: (f32, f32, f32, f32, f32)) -> Self {
		Self::new(other.0, other.1, other.2, other.3, other.4)
	}
}

struct BlockUv {
	umin: f32,
	umax: f32,
	vmin: f32,
	vmax: f32,
}

impl BlockUv {
	fn new_from_pos(x: u8, y: u8) -> Self {
		let x = (x as f32) / 16.0;
		let y = (y as f32) / 16.0;
		let s = 1.0 / 16.0;
		Self {
			umin: x,
			umax: x+s,
			vmin: y,
			vmax: y+s,
		}
	}
}
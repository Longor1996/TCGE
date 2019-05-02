//! Represents a simple prototypical 'game'-session.

use super::super::resources;
use super::super::router;
use super::render;
use super::geometry;
use super::freecam;
use crate::client::geometry::SimpleMesh;
use crate::client::geometry::SimpleMeshBuilder;
use rustc_hash::FxHashMap;
use super::cgmath::Matrix;

pub struct Scene {
	pub camera: freecam::Camera,
	meshes: Vec<geometry::SimpleMesh>,
	chunks: ChunkStorage,
}

impl Scene {
	pub fn new() -> Scene {
		let chunks = ChunkStorage::new();
		
		Scene {
			camera: freecam::Camera::new(),
			meshes: vec![
				// geometry::geometry_test(),
				// geometry::geometry_cube(1.0),
				// geometry::geometry_cube(-512.0),
			],
			chunks
		}
	}
	
}

impl router::comp::Component for Scene {
	fn get_type_name(&self) -> &'static str {
		"Scene"
	}
	
	fn on_attachment(&mut self, _node_id: usize) {}
	fn on_detachment(&mut self, _node_id: usize) {}
	
	fn on_load(&mut self) {}
	fn on_unload(&mut self) {}
	
	fn on_event(&mut self, _event: &mut router::event::Wrapper) {
		//
	}
}

type Block = u8;
const BLOCK_AIR: Block = 0;
const BLOCK_ADM: Block = 1;
const CHUNK_SIZE: usize = 16;
const CHUNK_SLICE: usize = CHUNK_SIZE*CHUNK_SIZE;
const CHUNK_VOLUME: usize = CHUNK_SLICE*CHUNK_SIZE;

#[derive(Eq, Hash, Clone)]
pub struct ChunkCoord {
	pub x: isize,
	pub y: isize,
	pub z: isize,
}

impl PartialEq for ChunkCoord {
	fn eq(&self, other: &ChunkCoord) -> bool {
		self.x == other.x
		&& self.y == other.y
		&& self.z == other.z
	}
}

pub struct Chunk {
	pub pos: ChunkCoord,
	pub blocks: [Block; CHUNK_VOLUME],
}

impl Chunk {
	
	pub fn new(x: isize, y: isize, z: isize) -> Chunk {
		let mut new = Chunk {
			pos: ChunkCoord {x,y,z},
			blocks: [0 as Block; CHUNK_VOLUME]
		};
		
		new.fill_with_noise(BLOCK_ADM, 0.1);
		new.fill_with_grid(BLOCK_ADM);
		
		new
	}
	
	fn clamp_chunk_coord(value: isize) -> Option<usize> {
		if value < 0 {
			return None
		}
		
		if value >= CHUNK_SIZE as isize {
			return None
		}
		
		return Some(value as usize)
	}
	
	pub fn fill_with_grid(&mut self, fill: Block) {
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
	
	pub fn fill_with_noise(&mut self, fill: Block, chance: f64) {
		extern crate rand;
		use rand::prelude::*;
		let mut rng = thread_rng();
		
		for i in self.blocks.iter_mut() {
			*i = if rng.gen_bool(chance) {fill} else {BLOCK_AIR};
		}
	}
	
	pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<Block> {
		let x = Chunk::clamp_chunk_coord(x)?;
		let y = Chunk::clamp_chunk_coord(y)?;
		let z = Chunk::clamp_chunk_coord(z)?;
		
		let index = y*CHUNK_SLICE + z*CHUNK_SIZE + x;
		unsafe {
			Some(*self.blocks.get_unchecked(index))
		}
	}
	
	pub fn set_block(&mut self, x: isize, y: isize, z: isize, state: Block) -> Option<()> {
		let x = Chunk::clamp_chunk_coord(x)?;
		let y = Chunk::clamp_chunk_coord(y)?;
		let z = Chunk::clamp_chunk_coord(z)?;
		
		let index = y*CHUNK_SLICE + z*CHUNK_SIZE + x;
		self.blocks[index] = state;
		Some(())
	}
	
	pub fn render_into_simple_mesh(&self) -> SimpleMesh {
		let mut builder = SimpleMeshBuilder::new();
		const N: f32 = 0.0;
		const S: f32 = 1.0;
		
		for y in 0..CHUNK_SIZE {
			for z in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					let x = x as isize;
					let y = y as isize;
					let z = z as isize;
					let block = self.get_block(x, y, z).unwrap_or(BLOCK_AIR);
					
					if block == BLOCK_AIR {
						continue;
					}
					
					let cbp = builder.current();
					
					if self.get_block(x,y+1,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // top
							N, S, S, // a
							S, S, S, // b
							S, S, N, // c
							N, S, N, // d
						]);
					}
					
					if self.get_block(x,y-1,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // bottom
							N, N, N, // d
							S, N, N, // c
							S, N, S, // b
							N, N, S, // a
						]);
					}
					
					if self.get_block(x,y,z-1).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // front
							N, S, N, // a
							S, S, N, // b
							S, N, N, // c
							N, N, N, // d
						]);
					}
					
					if self.get_block(x,y,z+1).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // back
							N, N, S, // d
							S, N, S, // c
							S, S, S, // b
							N, S, S, // a
						]);
					}
					
					if self.get_block(x-1,y,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // left
							N, S, S, // a
							N, S, N, // b
							N, N, N, // c
							N, N, S, // d
						]);
					}
					
					if self.get_block(x+1,y,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // right
							S, N, S, // d
							S, N, N, // c
							S, S, N, // b
							S, S, S, // a
						]);
					}
					
					builder.translate_range(cbp, None,
						(x + self.pos.x*CHUNK_SIZE as isize) as f32,
						(y + self.pos.y*CHUNK_SIZE as isize) as f32,
						(z + self.pos.z*CHUNK_SIZE as isize) as f32
					);
				}
			}
		}
		
		return builder.build();
	}
	
}

pub struct ChunkStorage {
	chunks: Vec<Chunk>
}

impl ChunkStorage {
	fn new() -> ChunkStorage {
		let mut storage = ChunkStorage {
			chunks: vec![]
		};
		
		for y in 0..3 {
			for z in 0..8 {
				for x in 0..8 {
					let chunk = Chunk::new(x, y, z);
					storage.chunks.push(chunk);
				}
			}
		}
		
		storage
	}
}

pub struct ChunkRenderManager {
	chunks: FxHashMap<ChunkCoord, SimpleMesh>,
	material: ShaderBlocks,
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


impl ChunkRenderManager {
	fn new(res: &resources::Resources) -> Result<ChunkRenderManager, render::utility::Error> {
		let material = ShaderBlocks::new(res)?;
		
		Ok(ChunkRenderManager {
			chunks: FxHashMap::default(),
			material,
		})
	}
	
	fn render(&mut self, scene: &Scene, transform: cgmath::Matrix4<f32>) {
		render::utility::gl_push_debug("chunks");
		
		self.material.shader.set_used();
		self.material.shader.uniform_matrix4(self.material.uniform_matrix, transform);
		self.material.shader.uniform_sampler(self.material.uniform_atlas, 0);
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.material.texatlas.id);
		}
		
		let mut max_uploads_per_frame: usize = 1;
		for chunk in scene.chunks.chunks.iter() {
			if self.chunks.contains_key(&chunk.pos) {
				self.chunks.get(&chunk.pos).unwrap().draw(gl::TRIANGLES);
			} else {
				if max_uploads_per_frame > 0 {
					max_uploads_per_frame -= 1;
					let mesh = chunk.render_into_simple_mesh();
					
					render::utility::gl_label_object(
						gl::VERTEX_ARRAY,
						mesh.get_gl_descriptor(),
						&format!("Chunk({}, {}, {}): Descriptor", chunk.pos.x, chunk.pos.y, chunk.pos.z)
					);
					
					render::utility::gl_label_object(
						gl::BUFFER,
						mesh.get_gl_vertex_buf(),
						&format!("Chunk({}, {}, {}): Geometry", chunk.pos.x, chunk.pos.y, chunk.pos.z)
					);
					
					self.chunks.insert(chunk.pos.clone(), mesh);
				}
			}
		}
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
		
		render::utility::gl_pop_debug();
	}
}

pub struct SceneRenderer {
	frame_id: i64,
	grid: render::grid::Grid,
	shader_random: render::materials::ShaderRandom,
	chunk_rmng: ChunkRenderManager,
}

impl SceneRenderer {
	pub fn new(res: &resources::Resources) -> Result<SceneRenderer, render::utility::Error> {
		let grid = render::grid::Grid::new(&res)?;
		let shader_random = render::materials::ShaderRandom::new(&res)?;
		let chunk_rmng = ChunkRenderManager::new(res)?;
		
		Ok(SceneRenderer {
			frame_id: 0,
			grid: grid,
			shader_random,
			chunk_rmng,
		})
	}
	
	pub fn begin(&mut self) {
		self.frame_id = self.frame_id + 1;
	}
	
	pub fn end(&mut self) {
		// ...?
	}
	
	pub fn reset(&mut self) {
		self.frame_id = 0;
	}
}

impl router::comp::Component for SceneRenderer {
	fn get_type_name(&self) -> &'static str {
		"SceneRenderState"
	}
	
	fn on_attachment(&mut self, _node_id: usize) {}
	fn on_detachment(&mut self, _node_id: usize) {}
	
	fn on_load(&mut self) {}
	fn on_unload(&mut self) {}
	
	fn on_event(&mut self, _event: &mut router::event::Wrapper) {
		//
	}
}

pub fn render(render_state: &mut SceneRenderer, scene: &Scene, size: (i32, i32), now: f64, interpolation:f32) {
	render::utility::gl_push_debug("Draw Scene");
	
	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::CullFace(gl::FRONT);
		gl::Enable(gl::CULL_FACE);
	}
	
	let camera = &scene.camera;
	
	let camera_position = camera.get_position(interpolation);
	let camera_transform = camera.transform(size, interpolation, true);
	
	render_state.grid.draw(&camera_transform, &camera_position);
	
	let shader_random = &render_state.shader_random;
	shader_random.shader_program.set_used();
	shader_random.shader_program.uniform_matrix4(shader_random.uniform_matrix, camera_transform);
	shader_random.shader_program.uniform_scalar(shader_random.uniform_time, now as f32);
	
	for mesh in scene.meshes.iter() {
		mesh.draw(gl::TRIANGLES);
	}
	
	// Render chunks!
	render_state.chunk_rmng.render(scene, camera_transform);
	
	render::utility::gl_pop_debug();
}

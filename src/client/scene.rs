//! Represents a simple prototypical 'game'-session.

use super::super::resources;
use super::super::router;
use super::render;
use super::geometry;
use super::freecam;
use crate::client::geometry::SimpleMesh;
use crate::client::geometry::SimpleMeshBuilder;

pub struct Scene {
	pub camera: freecam::Camera,
	meshes: Vec<geometry::SimpleMesh>,
}

impl Scene {
	pub fn new() -> Scene {
		let mut meshes = vec![
			geometry::geometry_test(),
			geometry::geometry_cube(1.0),
			// geometry::geometry_cube(-512.0),
		];
		
		for y in 0..=2 {
			for z in 0..=2 {
				for x in 0..=2 {
					let chunk = Chunk::new(x, y, z);
					let mesh = chunk.render_into_simple_mesh();
					meshes.push(mesh);
				}
			}
		}
		
		Scene {
			camera: freecam::Camera::new(),
			meshes: meshes
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
const CHUNK_SIZE: usize = 16;
const CHUNK_SLICE: usize = CHUNK_SIZE*CHUNK_SIZE;
const CHUNK_VOLUME: usize = CHUNK_SLICE*CHUNK_SIZE;

pub struct Chunk {
	pub x: isize,
	pub y: isize,
	pub z: isize,
	pub blocks: [Block; CHUNK_VOLUME],
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

impl Chunk {
	
	pub fn new(x: isize, y: isize, z: isize) -> Chunk {
		let mut new = Chunk {
			x, y, z,
			blocks: [0 as Block; CHUNK_VOLUME]
		};
		
		extern crate rand;
		use rand::prelude::*;
		let mut rng = thread_rng();
		
		for i in 0..new.blocks.len() {
			new.blocks[i] = if rng.gen::<u8>() > 200 {255} else {0};
		}
		
		new
	}
	
	pub fn get_block(&self, x: isize, y: isize, z: isize) -> Option<Block> {
		let x = clamp_chunk_coord(x)?;
		let y = clamp_chunk_coord(y)?;
		let z = clamp_chunk_coord(z)?;
		
		let index = y*CHUNK_SLICE + z*CHUNK_SIZE + x;
		match self.blocks.get(index) {
			Some(x) => Some(*x),
			None => None
		}
	}
	
	pub fn render_into_simple_mesh(&self) -> SimpleMesh {
		let mut builder = SimpleMeshBuilder::new();
		const S: f32 = 0.5;
		
		for y in 0..CHUNK_SIZE {
			for z in 0..CHUNK_SIZE {
				for x in 0..CHUNK_SIZE {
					let x = x as isize;
					let y = y as isize;
					let z = z as isize;
					let block = self.get_block(x, y, z).unwrap_or(BLOCK_AIR);
					
					if block < 127 {
						continue;
					}
					
					let cbp = builder.current();
					
					if self.get_block(x,y+1,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // top
							-S, S, S, // a
							S, S, S, // b
							S, S, -S, // c
							-S, S, -S, // d
						]);
					}
					
					if self.get_block(x,y-1,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // bottom
							-S, -S, -S, // d
							S, -S, -S, // c
							S, -S, S, // b
							-S, -S, S, // a
						]);
					}
					
					if self.get_block(x,y,z-1).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // front
							-S, S, -S, // a
							S, S, -S, // b
							S, -S, -S, // c
							-S, -S, -S, // d
						]);
					}
					
					if self.get_block(x,y,z+1).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // back
							-S, -S, S, // d
							S, -S, S, // c
							S, S, S, // b
							-S, S, S, // a
						]);
					}
					
					if self.get_block(x-1,y,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // left
							-S, S, S, // a
							-S, S, -S, // b
							-S, -S, -S, // c
							-S, -S, S, // d
						]);
					}
					
					if self.get_block(x+1,y,z).unwrap_or(BLOCK_AIR) == BLOCK_AIR {
						builder.push_quads(vec![ // right
							S, -S, S, // d
							S, -S, -S, // c
							S, S, -S, // b
							S, S, S, // a
						]);
					}
					
					builder.translate_range(cbp, None,
						(x + self.x*CHUNK_SIZE as isize) as f32,
						(y + self.y*CHUNK_SIZE as isize) as f32,
						(z + self.z*CHUNK_SIZE as isize) as f32
					);
				}
			}
		}
		
		return builder.build();
	}
	
}

pub struct SceneRenderer {
	frame_id: i64,
	grid: render::grid::Grid,
	shader_random: render::materials::ShaderRandom,
}

impl SceneRenderer {
	pub fn new(res: &resources::Resources) -> Result<SceneRenderer, render::utility::Error> {
		let grid = render::grid::Grid::new(&res)?;
		let shader_random = render::materials::ShaderRandom::new(&res)?;
		
		Ok(SceneRenderer {
			frame_id: 0,
			grid: grid,
			shader_random,
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

pub fn render(render_state: &SceneRenderer, scene: &Scene, size: (i32, i32), now: f64, interpolation:f32) {
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
	
	render::utility::gl_pop_debug();
}

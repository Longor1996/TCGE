use rustc_hash::FxHashMap;
use super::super::super::resources;
use super::super::super::blocks as blockdef;
use super::super::super::util::current_time_nanos;
use super::super::render;
use super::super::scene;

use super::ChunkCoord;
use super::mesher;

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
	
	chunks: FxHashMap<ChunkCoord, (u128, mesher::ChunkMeshState)>,
	material: ShaderBlocks,
}

impl ChunkRenderManager {
	pub fn new(res: &resources::Resources, blockdef: blockdef::UniverseRef) -> Result<ChunkRenderManager, render::utility::Error> {
		let material = ShaderBlocks::new(res)?;
		
		Ok(ChunkRenderManager {
			blockdef: blockdef.clone(),
			chunks: FxHashMap::default(),
			material,
		})
	}
	
	pub fn render(&mut self, scene: &scene::Scene, transform: cgmath::Matrix4<f32>) {
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
					*mesh = mesher::mesh(self.blockdef.clone(), &chunk);
				}
				
				if let mesher::ChunkMeshState::Meshed(mesh) = mesh {
					mesh.draw();
				}
				
			} else {
				if max_uploads_per_frame > 0 {
					max_uploads_per_frame -= 1;
					let mesh = mesher::mesh(self.blockdef.clone(), &chunk);
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

/// The graphical representation of a chunk.
/// Really just a bag of OpenGL Object-Handles.
pub struct ChunkMesh {
	descriptor: gl::types::GLuint,
	vertex_buf: gl::types::GLuint,
	count: i32,
}

impl ChunkMesh {
	pub fn new(descriptor: gl::types::GLuint, vertex_buf: gl::types::GLuint, count: i32) -> Self {
		Self {
			descriptor,
			vertex_buf,
			count
		}
	}
	
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

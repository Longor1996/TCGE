use rustc_hash::FxHashMap;
use super::super::super::resources;
use super::super::super::blocks as blockdef;
use super::super::super::util::current_time_nanos;
use super::super::render;
use super::super::scene;

use super::Chunk;
use super::ChunkCoord;
use super::CHUNK_SIZE;

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
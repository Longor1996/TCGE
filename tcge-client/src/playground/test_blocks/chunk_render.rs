use super::*;
use crate::playground::test_blocks::BlocksMaterialError;
use crate::render::{BufferObject, BufferObjectRef};

pub struct ChunkRenderManager {
	// Static
	gl: gl::Gl,
	blocks: BlocksRef,
	material: BlocksMaterial,
	bakery: StaticBlockBakery,
	qindex: BufferObjectRef,
	
	// Dynamic
	chunks: FxHashMap<ChunkCoord, (u128, ChunkMeshState)>,
	mesher: MesherThreadState,
	
	// Per Frame
	calls: Vec<(gl::types::GLuint, gl::types::GLsizei)>,
}

impl ChunkRenderManager {
	pub fn new(
		gl: &gl::Gl,
		res: &resources::Resources,
		blocks: &BlocksRef
	) -> Result<Self, BlocksMaterialError> {
		
		let material = BlocksMaterial::new(gl, res)?;
		let bakery = StaticBlockBakery::new(&res, &blocks).unwrap();
		let qindex = Self::generate_quad_indices(gl, 4096).to_ref();
		
		gl.label_object(
			gl::BUFFER,
			qindex.id,
			"Quads Index"
		);
		
		Ok(Self {
			gl: gl.clone(),
			blocks: blocks.clone(),
			material,
			bakery,
			qindex,
			chunks: FxHashMap::default(),
			mesher: MesherThreadState::new(),
			calls: vec![],
		})
	}
	
	pub fn generate_quad_indices(gl: &gl::Gl, max: usize) -> BufferObject {
		let mut indices: Vec<u16> = vec![];
		for i in 0..max {
			// A: a b d
			// B: b c d
			let o = i as u16 * 4;
			indices.append(&mut vec![
				o+0, o+1, o+3,
				o+1, o+2, o+3,
			]);
		}
		
		trace!("Allocating index buffer...");
		BufferObject::buffer_data(
			&gl,
			gl::ELEMENT_ARRAY_BUFFER,
			gl::DYNAMIC_DRAW,
			&indices
		)
	}
	
	pub fn render(&mut self, chunks: &ChunkStorage, transform: &cgmath::Matrix4<f32>) {
		self.gl.push_debug("Chunks");
		
		use cgmath::InnerSpace;
		let sun = cgmath::Vector3::new(0.9, 1.0, 0.7).normalize();
		
		self.material.shader.set_used();
		self.material.shader.set_uniform_matrix4(self.material.uniform_matrix, transform);
		self.material.shader.set_uniform_vector3(self.material.uniform_sun, &sun);
		self.material.shader.set_uniform_sampler(self.material.uniform_atlas, 0);
		self.material.atlas.set_used();
		
		self.gl.push_debug("Chunk-Uploads");
		
		let mut max_uploads_per_frame: usize = 2;
		for (cpos, chunk) in chunks.chunks.iter() {
			
			if self.chunks.contains_key(cpos) {
				let (time, mesh) = self.chunks.get_mut(cpos).unwrap();
				
				if chunk.last_update > *time && max_uploads_per_frame > 0 {
					max_uploads_per_frame -= 1;
					
					let block_data = chunks.get_chunk_with_edges(cpos).unwrap();
					
					*time = chunk.last_update;
					
					let ptree = common::profiler::profiler().get_current();
					
					ptree.enter_noguard("mesh-chunk");
					mesh_chunk(
						&mut self.mesher,
						self.blocks.clone(),
						&self.bakery,
						&chunk,
						&block_data
					);
					
					*mesh = upload(&self.gl, &chunk.pos, &self.mesher.vertices, &self.qindex);
					
					ptree.leave();
				}
				
				if let ChunkMeshState::Meshed(mesh) = mesh {
					self.calls.push(mesh.draw_later());
				}
			} else {
				if max_uploads_per_frame > 0 {
					max_uploads_per_frame -= 1;
					
					let block_data = chunks.get_chunk_with_edges(cpos).unwrap();
					
					let ptree = common::profiler::profiler().get_current();
					ptree.enter_noguard("mesh-chunk");
					
					mesh_chunk(
						&mut self.mesher,
						self.blocks.clone(),
						&self.bakery,
						&chunk,
						&block_data
					);
					
					let mesh = upload(&self.gl, &chunk.pos, &self.mesher.vertices, &self.qindex);
					
					if let ChunkMeshState::Meshed(mesh) = &mesh {
						self.calls.push(mesh.draw_later());
					}
					
					ptree.leave();
					
					self.chunks.insert(cpos.clone(), (current_time_nanos(), mesh));
				}
			}
		}
		
		self.gl.pop_debug();
		
		self.gl.clone().scope_debug("Chunk-Draws", &mut || {
			// TODO: Optimize with https://www.reddit.com/r/opengl/comments/3m9u36/how_to_render_using_glmultidrawarraysindirect/
			while let Some(chunk_mesh_raw) = self.calls.pop() {
				draw_chunk(&self.gl, chunk_mesh_raw);
			}
		});
		
		self.gl.pop_debug();
	}
}

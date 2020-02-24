use super::*;
use crate::playground::test_blocks::BlocksMaterialError;
use crate::render::{BufferObject, BufferObjectRef};

pub struct ChunkRenderManager {
	// Static
	gl: gl::Gl,
	blocks: BlocksRef,
	material: BlocksMaterial,
	bakery: StaticBlockBakeryRef,
	quad_index: BufferObjectRef,
	
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
		blocks: &BlocksRef,
		bakery: StaticBlockBakeryRef
	) -> Result<Self, BlocksMaterialError> {
		
		let material = BlocksMaterial::new(gl, res)?;
		let quad_index = Self::generate_quad_indices(gl, 4096).to_ref();
		
		gl.label_object(
			gl::BUFFER,
			quad_index.id,
			"Quads Index"
		);
		
		Ok(Self {
			gl: gl.clone(),
			blocks: blocks.clone(),
			material,
			bakery,
			quad_index,
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
	
	pub fn render(&mut self, chunks: &ChunkStorage, transform: &nalgebra_glm::Mat4) {
		self.gl.push_debug("Chunks");
		
		let sun = nalgebra_glm::Vec3::new(0.9, 1.0, 0.7).normalize();
		
		self.material.shader.set_used();
		self.material.shader.set_uniform_matrix4(self.material.uniform_matrix, transform);
		self.material.shader.set_uniform_vector3(self.material.uniform_sun, &sun);
		self.material.shader.set_uniform_sampler(self.material.uniform_atlas, 0);
		self.material.atlas.set_used();
		
		self.gl.push_debug("Chunk-Uploads");
		
		let mut max_uploads_per_frame: usize = 2;
		for (chunk_pos, chunk) in chunks.chunks.iter() {
			
			if self.chunks.contains_key(chunk_pos) {
				let (time, mesh) = self.chunks.get_mut(chunk_pos).unwrap();
				
				if chunk.last_update > *time && max_uploads_per_frame > 0 {
					max_uploads_per_frame -= 1;
					
					let block_data = chunks.get_chunk_with_edges(chunk_pos).unwrap();
					
					*time = chunk.last_update;
					
					let profiler_tree = common::profiler::profiler().get_current();
					
					profiler_tree.enter_noguard("mesh-chunk");
					mesh_chunk(
						&mut self.mesher,
						self.blocks.clone(),
						&self.bakery,
						&chunk,
						&block_data
					);
					
					*mesh = upload(&self.gl, &chunk.pos, &self.mesher.vertices, &self.quad_index);
					
					profiler_tree.leave();
				}
				
				if let ChunkMeshState::Meshed(mesh) = mesh {
					self.calls.push(mesh.draw_later());
				}
			} else if max_uploads_per_frame > 0 {
				max_uploads_per_frame -= 1;
				
				let block_data = chunks.get_chunk_with_edges(chunk_pos).unwrap();
				
				let profiler_tree = common::profiler::profiler().get_current();
				profiler_tree.enter_noguard("mesh-chunk");
				
				mesh_chunk(
					&mut self.mesher,
					self.blocks.clone(),
					&self.bakery,
					&chunk,
					&block_data
				);
				
				let mesh = upload(&self.gl, &chunk.pos, &self.mesher.vertices, &self.quad_index);
				
				if let ChunkMeshState::Meshed(mesh) = &mesh {
					self.calls.push(mesh.draw_later());
				}
				
				profiler_tree.leave();
				
				self.chunks.insert(chunk_pos.clone(), (current_time_nanos(), mesh));
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

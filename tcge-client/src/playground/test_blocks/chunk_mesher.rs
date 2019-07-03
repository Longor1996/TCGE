use super::*;
use crate::render;

/// The graphical state of a chunk.
pub enum ChunkMeshState {
	/// Chunk is meshed but empty.
	Empty,
	
	/// Chunk is meshed and full.
	Meshed(ChunkMesh),
}

/// The graphical representation of a chunk.
/// Really just a bag of OpenGL Object-Handles.
pub struct ChunkMesh {
	gl: gl::Gl,
	descriptor: gl::types::GLuint,
	vertices: render::BufferObject,
	count: i32,
}

impl ChunkMesh {
	pub fn new(gl: &gl::Gl, descriptor: gl::types::GLuint, vertices: render::BufferObject, count: i32) -> Self {
		Self {
			gl: gl.clone(),
			descriptor,
			vertices,
			count
		}
	}
	
	pub fn draw(&self) {
		unsafe {
			self.gl.BindVertexArray(self.descriptor);
			self.gl.DrawElements(
				gl::TRIANGLES,
				self.count,
				gl::UNSIGNED_SHORT,
				0 as *const gl::types::GLvoid
			);
		}
	}
}

impl Drop for ChunkMesh {
	fn drop(&mut self) {
		unsafe {
			let tmp = [self.vertices.id];
			self.gl.DeleteBuffers(1, tmp.as_ptr());
			
			let tmp = [self.descriptor];
			self.gl.DeleteVertexArrays(1, tmp.as_ptr());
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
	
	// AO
	pub ao: f32,
}

impl ChunkMeshVertex {
	pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, ao: f32) -> Self {
		Self {
			x, y, z, u, v, ao
		}
	}
}

impl From<(f32, f32, f32, f32, f32)> for ChunkMeshVertex {
	fn from(other: (f32, f32, f32, f32, f32)) -> Self {
		Self::new(other.0, other.1, other.2, other.3, other.4, 0.0)
	}
}

impl From<BakedBlockMeshVertex> for ChunkMeshVertex {
	fn from(other: BakedBlockMeshVertex) -> Self {
		Self::new(other.x, other.y, other.z, other.u, other.v, 0.0)
	}
}

impl From<&BakedBlockMeshVertex> for ChunkMeshVertex {
	fn from(other: &BakedBlockMeshVertex) -> Self {
		Self::new(other.x, other.y, other.z, other.u, other.v, 0.0)
	}
}

pub struct MesherThreadState {
	vertices: Vec<ChunkMeshVertex>,
	quad_buf: Vec<BakedBlockMeshVertex>
}

impl MesherThreadState {
	pub fn new() -> MesherThreadState {
		MesherThreadState {
			vertices: vec![],
			quad_buf: vec![],
		}
	}
	
	pub fn reset(&mut self) {
		self.vertices.clear();
		self.quad_buf.clear();
	}
}


pub fn mesh_chunk(
	gl: &gl::Gl,
	qindex: &render::BufferObject,
	mesher: &mut MesherThreadState,
	blocks: BlocksRef,
	static_bakery: &StaticBlockBakery,
	chunk: &Chunk,
	neighbours: &[Option<&Chunk>; 27]
) -> ChunkMeshState {
	let start = current_time_nanos();
	
	// --- Reset state of the mesher, clearing the buffers.
	mesher.reset();
	let vertices = &mut mesher.vertices;
	
	let air = blocks
		.get_block_by_name_unchecked("air")
		.get_default_state();
	
	let (cx, cy, cz) = chunk.pos.to_block_coord();
	
	// --- Local function for fetching blocks quickly...
	let get_block = |
		offset: &BlockCoord,
	| {
		if chunk.pos.contains_block(offset) {
			return Some(unsafe {
				chunk.get_block_unchecked(offset.x, offset.y, offset.z)
			})
		}
		
		let o_cpos = ChunkCoord::new_from_block(offset);
		for o_chunk in neighbours.iter() {
			if let Some(o_chunk) = o_chunk {
				if o_chunk.pos == o_cpos {
					return Some(unsafe {
						o_chunk.get_block_unchecked(offset.x, offset.y, offset.z)
					})
				}
			}
		}
		
		None
	};
	
	let mut block_pos = BlockCoord::new(0, 0, 0);
	let mut context = BakeryContext::new();
	
	for y in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				let x = x as BlockDim;
				let y = y as BlockDim;
				let z = z as BlockDim;
				let block = unsafe {chunk.get_block_unchecked(x, y, z)};
				
				if block == air {
					continue;
				}
				
				let cbx = x + cx;
				let cby = y + cy;
				let cbz = z + cz;
				block_pos.set(cbx, cby, cbz);
				
				let cbp = vertices.len();
				
				context.set_occlusion(
					get_block(&block_pos.right   (1)).unwrap_or(air) != air,
					get_block(&block_pos.up      (1)).unwrap_or(air) != air,
					get_block(&block_pos.backward(1)).unwrap_or(air) != air,
					get_block(&block_pos.left    (1)).unwrap_or(air) != air,
					get_block(&block_pos.down    (1)).unwrap_or(air) != air,
					get_block(&block_pos.forward (1)).unwrap_or(air) != air,
					true
				);
				
				static_bakery.render_block(&context, &block, &mut |face| {
					vertices.push(face[0].into());
					vertices.push(face[1].into());
					vertices.push(face[2].into());
					vertices.push(face[3].into());
				});
				
				let cbx = cbx as f32;
				let cby = cby as f32;
				let cbz = cbz as f32;
				for vertex in &mut vertices[cbp..] {
					vertex.ao = 0.0;
					
					vertex.x += cbx;
					vertex.y += cby;
					vertex.z += cbz;
				}
			}
		}
	}
	
	let end = current_time_nanos();
	let duration = end - start;
	if duration > 100 {
		debug!("Took {}ns to mesh chunk {}.", duration, chunk.pos);
	}
	
	return upload(gl, chunk, &vertices, &qindex);
}

fn upload(gl: &gl::Gl, chunk: &Chunk, mesh_data: &Vec<ChunkMeshVertex>, qindex: &render::BufferObject) -> ChunkMeshState {
	// Don't upload empty meshes.
	if mesh_data.len() == 0 {
		return ChunkMeshState::Empty
	}
	
	let vertex_count = mesh_data.len() / 4 * 6;
	
	let vbo = render::BufferObject::buffer_data(gl, gl::ARRAY_BUFFER, gl::STATIC_DRAW, mesh_data);
	
	let mut vao: gl::types::GLuint = 0;
	unsafe {
		gl.GenVertexArrays(1, &mut vao);
		gl.BindVertexArray(vao);
		gl.BindBuffer(gl::ARRAY_BUFFER, vbo.id);
		
		// Bind the index buffer
		gl.BindBuffer(qindex.target, qindex.id);
		
		gl.EnableVertexAttribArray(0);
		gl.VertexAttribPointer(
			0, // attribute location
			3, // sub-element count
			gl::FLOAT, // sub-element type
			gl::FALSE, // sub-element normalization
			(6 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
			(0 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
		);
		
		gl.EnableVertexAttribArray(1);
		gl.VertexAttribPointer(
			1, // attribute location
			2, // sub-element count
			gl::FLOAT, // sub-element type
			gl::FALSE, // sub-element normalization
			(6 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
			(3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
		);
		
		gl.EnableVertexAttribArray(2);
		gl.VertexAttribPointer(
			2, // attribute location
			1, // sub-element count
			gl::FLOAT, // sub-element type
			gl::FALSE, // sub-element normalization
			(6 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
			(5 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
		);
		
		gl.BindVertexArray(0);
	}
	
	let label = format!("Chunk({}, {}, {})", chunk.pos.x, chunk.pos.y, chunk.pos.z);
	
	gl.label_object(
		gl::VERTEX_ARRAY, vao,
		&format!("{} Descriptor", label)
	);
	
	gl.label_object(
		gl::BUFFER, vbo.id,
		&format!("{} Geometry", label)
	);
	
	ChunkMeshState::Meshed(ChunkMesh::new(
		gl,
		vao,
		vbo,
		vertex_count as i32
	))
}

fn lerp_trilinear(x: f32, y: f32, z: f32, corners: &[f32; 8]) -> f32 {
	(1.0 - x) * (1.0 - y) * (1.0 - z) * corners[0] +
		x * (1.0 - y) * (1.0 - z) * corners[1] +
		(1.0 - x) * y * (1.0 - z) * corners[2] +
		x * y * (1.0 - z) * corners[3] +
		(1.0 - x) * (1.0 - y) * z * corners[4] +
		x * (1.0 - y) * z * corners[5] +
		(1.0 - x) * y * z * corners[6] +
		x * y * z * corners[7]
}

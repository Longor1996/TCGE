use crate::blocks as blockdef;
use crate::blocks::BlockCoord;
use crate::client::blocks::ChunkCoord;
use super::super::render::utility::gl_label_object;
use super::static_bakery;
use super::static_bakery::BakedBlockMeshVertex;
use super::render;
use super::Chunk;
use super::CHUNK_SIZE;

/// The graphical state of a chunk.
pub enum ChunkMeshState {
	/// Chunk is meshed but empty.
	Empty,
	
	/// Chunk is meshed and full.
	Meshed(render::ChunkMesh),
}

pub struct MesherThreadState {
	vertices: Vec<ChunkMeshVertex>,
	quad_buf: Vec<BakedBlockMeshVertex>
}

impl MesherThreadState {
	pub fn new() -> MesherThreadState{
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

pub fn mesh(
	mesher: &mut MesherThreadState,
	blockdef: blockdef::UniverseRef,
	static_bakery: &static_bakery::StaticBlockBakery,
	chunk: &Chunk,
	neighbours: &[Option<&Chunk>; 27]
) -> ChunkMeshState {
	let start = crate::util::current_time_nanos();
	
	// --- Reset state of the mesher, clearing the buffers.
	mesher.reset();
	let vertices = &mut mesher.vertices;
	let mut quad_buf = &mut mesher.quad_buf;
	
	let air = blockdef
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
	let mut context = static_bakery::BakeryContext::new();
	
	let mut TP_OCCLUSION = crate::util::TinyProfiler::new();
	let mut TP_GET_BAKED = crate::util::TinyProfiler::new();
	let mut TP_TESSELLATE = crate::util::TinyProfiler::new();
	let mut TP_TRANSLATE = crate::util::TinyProfiler::new();
	
	for y in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			for x in 0..CHUNK_SIZE {
				let x = x as isize;
				let y = y as isize;
				let z = z as isize;
				let block = unsafe {chunk.get_block_unchecked(x, y, z)};
				
				if block == air {
					continue;
				}
				
				let cbx = x + cx;
				let cby = y + cy;
				let cbz = z + cz;
				block_pos.set(cbx, cby, cbz);
				
				let cbp = vertices.len();
				
				TP_OCCLUSION.start();
				context.set_occlusion(
					get_block(&block_pos.right   (1)).unwrap_or(air) != air,
					get_block(&block_pos.up      (1)).unwrap_or(air) != air,
					get_block(&block_pos.backward(1)).unwrap_or(air) != air,
					get_block(&block_pos.left    (1)).unwrap_or(air) != air,
					get_block(&block_pos.down    (1)).unwrap_or(air) != air,
					get_block(&block_pos.forward (1)).unwrap_or(air) != air,
					true
				);
				TP_OCCLUSION.end();
				
				TP_GET_BAKED.start();
				quad_buf.clear();
				static_bakery.render_block(&context, &block, &mut quad_buf);
				TP_GET_BAKED.end();
				
				TP_TESSELLATE.start();
				for quad in quad_buf.chunks_exact(4) {
					vertices.reserve(6);
					vertices.push(quad[0].into()); // a
					vertices.push(quad[1].into()); // b
					vertices.push(quad[3].into()); // d
					vertices.push(quad[1].into()); // b
					vertices.push(quad[2].into()); // c
					vertices.push(quad[3].into()); // d
				}
				TP_TESSELLATE.end();
				
				TP_TRANSLATE.start();
				let cbx = cbx as f32;
				let cby = cby as f32;
				let cbz = cbz as f32;
				for vertex in &mut vertices[cbp..] {
					vertex.x += cbx;
					vertex.y += cby;
					vertex.z += cbz;
				}
				TP_TRANSLATE.end();
			}
		}
	}
	
	let end = crate::util::current_time_nanos();
	let duration = end - start;
	if duration > 100 {
		debug!("Took {}ns to mesh chunk {}.", duration, chunk.pos);
		debug!("-- OCCLUSION: {}ns / {}ns", TP_OCCLUSION.average(), TP_OCCLUSION.total());
		debug!("-- GET_BAKED: {}ns / {}ns", TP_GET_BAKED.average(), TP_GET_BAKED.total());
		debug!("-- TRIANGLES: {}ns / {}ns", TP_TESSELLATE.average(), TP_TESSELLATE.total());
		debug!("-- TRANSLATE: {}ns / {}ns", TP_TRANSLATE.average(), TP_TRANSLATE.total());
	}
	
	return upload(chunk, &vertices);
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
	
	gl_label_object(
		gl::VERTEX_ARRAY, vao,
		&format!("{} Descriptor", label)
	);
	
	gl_label_object(
		gl::BUFFER, vbo,
		&format!("{} Geometry", label)
	);
	
	return ChunkMeshState::Meshed(render::ChunkMesh::new(
		vao,
		vbo,
		vertex_count as i32
	))
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

impl From<BakedBlockMeshVertex> for ChunkMeshVertex {
	fn from(other: BakedBlockMeshVertex) -> Self {
		Self::new(other.x, other.y, other.z, other.u, other.v)
	}
}

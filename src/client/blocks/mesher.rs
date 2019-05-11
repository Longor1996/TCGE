use crate::blocks as blockdef;
use super::super::render::utility::gl_label_object;
use super::render;
use super::Chunk;
use super::CHUNK_SIZE;
use super::CHUNK_SIZE_MASK;
use crate::client::blocks::ChunkCoord;
use crate::blocks::BlockCoord;

/// The graphical state of a chunk.
pub enum ChunkMeshState {
	/// Chunk is meshed but empty.
	Empty,
	
	/// Chunk is meshed and full.
	Meshed(render::ChunkMesh),
}


fn get_block(neighbours: &[Option<&Chunk>; 27], pos: &BlockCoord) -> Option<blockdef::BlockState> {
	let cpos = ChunkCoord::new_from_block(pos);
	let csm = CHUNK_SIZE_MASK as isize;
	
	for chunk in neighbours.iter() {
		if let Some(chunk) = chunk {
			if chunk.pos == cpos {
				let cx = pos.x & csm;
				let cy = pos.y & csm;
				let cz = pos.z & csm;
				if let Some(block) = chunk.get_block(cx,cy,cz) {
					return Some(block)
				}
			}
		}
	}
	
	None
}

pub fn mesh(blockdef: blockdef::UniverseRef, chunk: &Chunk, neighbours: &[Option<&Chunk>; 27]) -> ChunkMeshState {
	let mut vertices: Vec<ChunkMeshVertex> = vec![];
	
	let cpos = chunk.pos;
	
	const N: f32 = 0.0;
	const S: f32 = 1.0;
	
	let air = blockdef
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
				
				let cbx = x + cpos.x*CHUNK_SIZE as isize;
				let cby = y + cpos.y*CHUNK_SIZE as isize;
				let cbz = z + cpos.z*CHUNK_SIZE as isize;
				let pos = BlockCoord::new(cbx,cby,cbz);
				
				let cbp = vertices.len();
				
				// This line is the dumbest thing in the whole project...
				let uv = BlockUv::new_from_pos(block.id.get_raw_id() as u8 - 1, 0);
				// TODO: Implement the static block-bakery.
				
				if get_block(neighbours, &pos.up(1)).unwrap_or(air) == air {
					quad_to_tris(&[ // top
						(N, S, S, uv.umin, uv.vmin).into(),
						(S, S, S, uv.umax, uv.vmin).into(),
						(S, S, N, uv.umax, uv.vmax).into(),
						(N, S, N, uv.umin, uv.vmax).into(),
					], &mut vertices);
				}
				
				if get_block(neighbours, &pos.down(1)).unwrap_or(air) == air {
					quad_to_tris(&[ // bottom
						(N, N, N, uv.umin, uv.vmin).into(),
						(S, N, N, uv.umax, uv.vmin).into(),
						(S, N, S, uv.umax, uv.vmax).into(),
						(N, N, S, uv.umin, uv.vmax).into(),
					], &mut vertices);
				}
				
				if get_block(neighbours, &pos.forward(1)).unwrap_or(air) == air {
					quad_to_tris(&[ // front
						(N, S, N, uv.umin, uv.vmin).into(),
						(S, S, N, uv.umax, uv.vmin).into(),
						(S, N, N, uv.umax, uv.vmax).into(),
						(N, N, N, uv.umin, uv.vmax).into(),
					], &mut vertices);
				}
				
				if get_block(neighbours, &pos.backward(1)).unwrap_or(air) == air {
					quad_to_tris(&[ // back
						(N, N, S, uv.umin, uv.vmin).into(),
						(S, N, S, uv.umax, uv.vmin).into(),
						(S, S, S, uv.umax, uv.vmax).into(),
						(N, S, S, uv.umin, uv.vmax).into(),
					], &mut vertices);
				}
				
				if get_block(neighbours, &pos.left(1)).unwrap_or(air) == air {
					quad_to_tris(&[ // left
						(N, S, S, uv.umin, uv.vmin).into(),
						(N, S, N, uv.umax, uv.vmin).into(),
						(N, N, N, uv.umax, uv.vmax).into(),
						(N, N, S, uv.umin, uv.vmax).into(),
					], &mut vertices);
				}
				
				if get_block(neighbours, &pos.right(1)).unwrap_or(air) == air {
					quad_to_tris(&[ // right
						(S, N, S, uv.umin, uv.vmin).into(),
						(S, N, N, uv.umax, uv.vmin).into(),
						(S, S, N, uv.umax, uv.vmax).into(),
						(S, S, S, uv.umin, uv.vmax).into(),
					], &mut vertices);
				}
				
				for vertex in &mut vertices[cbp..] {
					vertex.x += cbx as f32;
					vertex.y += cby as f32;
					vertex.z += cbz as f32;
				}
			}
		}
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

fn quad_to_tris(src: &[ChunkMeshVertex; 4], dst: &mut Vec<ChunkMeshVertex>) {
	dst.reserve(6);
	dst.push(src[0]);
	dst.push(src[1]);
	dst.push(src[3]);
	dst.push(src[1]);
	dst.push(src[2]);
	dst.push(src[3]);
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

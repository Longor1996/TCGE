use super::gl::Gl;
use std::ffi::{CString};

////////////////////////////////////////////////////////////////////////////////
// --- 'Generic' submodules and their exports...

pub mod shader_object;
pub use shader_object::ShaderObject;
pub use shader_object::ShaderObjectRef;

pub mod program_object;
pub use program_object::ProgramObject;
pub use program_object::UniformLocation;

pub mod buffer_object;
pub use buffer_object::BufferObject;
pub use buffer_object::BufferObjectRef;

pub mod texture_object;
pub use texture_object::TextureObject;
pub use texture_object::TextureObjectBuilder;
pub use texture_object::TextureError;

pub mod vertex_array;
pub use vertex_array::VertexArray;
pub use vertex_array::VertexArrayAttrib;
pub use vertex_array::VertexArrayBuilder;

pub mod camera;
pub use camera::Camera;

// --- 'Special' submodules...
pub mod text;
pub mod geometry;
pub mod materials;
pub mod wireframe;

////////////////////////////////////////////////////////////////////////////////
// --- Utility Functions

pub fn create_whitespace_cstring_with_len(len: usize) -> CString {
	let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
	buffer.extend([b' '].iter().cycle().take(len as usize));
	unsafe { CString::from_vec_unchecked(buffer) }
}

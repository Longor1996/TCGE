use super::Gl;
use std::ffi::CStr;
use std::rc::Rc;

pub type ShaderObjectRef = Rc<ShaderObject>;

#[allow(dead_code)]
pub const SHADER_FILE_EXTENSIONS: [(&str, gl::types::GLenum); 4] = [
	(".vert", gl::VERTEX_SHADER),
	(".frag", gl::FRAGMENT_SHADER),
	(".geom", gl::GEOMETRY_SHADER),
	(".comp", gl::GEOMETRY_SHADER),
];

pub struct ShaderObject {
	pub gl: Gl,
	pub id: gl::types::GLuint,
	pub kind: gl::types::GLenum,
}

impl ShaderObject {
	#[allow(dead_code)]
	pub fn new_vertex_shader(gl: &Gl, source: &CStr) -> Result<ShaderObjectRef, String> {
		Self::new_shader(gl, source, gl::VERTEX_SHADER)
	}
	
	#[allow(dead_code)]
	pub fn new_fragment_shader(gl: &Gl, source: &CStr) -> Result<ShaderObjectRef, String> {
		Self::new_shader(gl, source, gl::FRAGMENT_SHADER)
	}
	
	#[allow(dead_code)]
	pub fn new_geometry_shader(gl: &Gl, source: &CStr) -> Result<ShaderObjectRef, String> {
		Self::new_shader(gl, source, gl::GEOMETRY_SHADER)
	}
	
	#[allow(dead_code)]
	pub fn new_compute_shader(gl: &Gl, source: &CStr) -> Result<ShaderObjectRef, String> {
		Self::new_shader(gl, source, gl::COMPUTE_SHADER)
	}
	
	#[allow(dead_code)]
	pub fn new_shader(gl: &Gl, source: &CStr, kind: gl::types::GLuint) -> Result<ShaderObjectRef, String> {
		let id = unsafe { gl.CreateShader(kind) };
		
		unsafe {
			gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
			gl.CompileShader(id);
		}
		
		let mut success: gl::types::GLint = 1;
		unsafe {
			gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
		}
		
		if success == 0 {
			let mut len: gl::types::GLint = 0;
			unsafe {
				gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
			}
			
			let error = super::create_whitespace_cstring_with_len(len as usize);
			unsafe {
				gl.GetShaderInfoLog(
					id, len,
					std::ptr::null_mut(),
					error.as_ptr() as *mut gl::types::GLchar
				);
			}
			
			return Err(error.to_string_lossy().into_owned());
		}
		
		Ok(Rc::new(ShaderObject {
			gl: gl.clone(),
			id, kind
		}))
	}
	
	#[allow(dead_code)]
	pub fn get_file_extension(&self) -> &'static str {
		SHADER_FILE_EXTENSIONS.iter()
			.find(|&&(_, shader_kind)| self.kind == shader_kind)
			.map(|&(file_ext, _)| file_ext)
			.expect("Unknown Shader Kind")
	}
	
}

impl Drop for ShaderObject {
	fn drop(&mut self) {
		unsafe {
			self.gl.DeleteShader(self.id);
		}
	}
}

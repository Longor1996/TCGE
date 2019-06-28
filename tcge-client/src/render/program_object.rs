use super::Gl;
use super::ShaderObject;
use super::ShaderObjectRef;
use std::rc::Rc;
use std::ffi::CString;
use cgmath::{Matrix, Array};

pub type UniformLocation = gl::types::GLint;

#[allow(dead_code)]
pub struct ProgramObject {
	pub gl: Gl,
	pub id: gl::types::GLuint,
	pub name: String,
	pub shaders: smallvec::SmallVec<[Rc<ShaderObject>; 2]>,
}

impl ProgramObject {
	#[allow(dead_code)]
	pub fn new(gl: &Gl, name: &str, shaders: &smallvec::SmallVec<[ShaderObjectRef;2]>) -> Result<ProgramObject, String>  {
		debug!("Creating & linking program: {}", name);
		
		let id = unsafe { gl.CreateProgram() };
		
		for shader in shaders {
			unsafe {
				gl.AttachShader(id, shader.id);
				
				gl.label_object(
					gl::SHADER,
					shader.id,
					&format!("Shader: {}{}", name, shader.get_file_extension())
				);
			}
		}
		
		unsafe { gl.LinkProgram(id); }
		
		let mut success: gl::types::GLint = 1;
		unsafe {
			gl.GetProgramiv(id, gl::LINK_STATUS, &mut success);
		}
		
		if success == 0 {
			let mut len: gl::types::GLint = 0;
			unsafe {
				gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
			}
			
			let error = super::create_whitespace_cstring_with_len(len as usize);
			unsafe {
				gl.GetProgramInfoLog(
					id,
					len,
					std::ptr::null_mut(),
					error.as_ptr() as *mut gl::types::GLchar
				);
			}
			
			return Err(error.to_string_lossy().into_owned());
		}
		
		gl.label_object(
			gl::PROGRAM,
			id,
			&format!("Program: {}", name)
		);
		
		Ok(ProgramObject {
			gl: gl.clone(),
			id,
			name: name.to_string(),
			shaders: shaders.clone()
		})
	}
	
	#[allow(dead_code)]
	pub fn set_used(&self) {
		unsafe {
			self.gl.UseProgram(self.id);
		}
	}
}

// Code that deals with uniforms.
impl ProgramObject {
	#[allow(dead_code)]
	pub fn get_uniform_location(&self, uniform_name: &str) -> Result<UniformLocation, ()> {
		let uniform_name_c = CString::new(uniform_name).unwrap();
		unsafe {
			let loc = self.gl.GetUniformLocation(self.id, uniform_name_c.as_ptr());
			
			if loc == -1 {
				warn!("Uniform location for '{}' in '{}' is invalid.", uniform_name, self.name);
				return Err(());
			}
			
			Ok(loc)
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_sampler(&self, uniform: UniformLocation, value: gl::types::GLuint) -> bool {
		if uniform == -1 {return false}
		unsafe {
			self.gl.Uniform1i(uniform, value as i32);
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_scalar(&self, uniform: UniformLocation, value: f32) -> bool {
		if uniform == -1 {return false}
		unsafe {
			self.gl.Uniform1f(uniform, value);
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_vector2(&self, uniform: UniformLocation, vector: &cgmath::Vector2<f32>) -> bool {
		if uniform == -1 {return false}
		unsafe {
			self.gl.Uniform2fv(uniform, 1, vector.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_vector2_raw(&self, uniform: UniformLocation, vector: &[f32]) -> bool {
		if uniform == -1 {return false}
		if vector.len() != 2 {return false}
		unsafe {
			self.gl.Uniform2fv(uniform, 1, vector.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_vector3(&self, uniform: UniformLocation, vector: &cgmath::Vector3<f32>) -> bool {
		if uniform == -1 {return false}
		unsafe {
			self.gl.Uniform3fv(uniform, 1, vector.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_vector3_raw(&self, uniform: UniformLocation, vector: &[f32]) -> bool {
		if uniform == -1 {return false}
		if vector.len() != 3 {return false}
		unsafe {
			self.gl.Uniform3fv(uniform, 1, vector.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_vector4(&self, uniform: UniformLocation, vector: &cgmath::Vector4<f32>) -> bool {
		if uniform == -1 {return false}
		unsafe {
			self.gl.Uniform4fv(uniform, 1, vector.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_vector4_raw(&self, uniform: UniformLocation, vector: &[f32]) -> bool {
		if uniform == -1 {return false}
		if vector.len() != 4 {return false}
		unsafe {
			self.gl.Uniform4fv(uniform, 1, vector.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_matrix4(&self, uniform: UniformLocation, matrix: &cgmath::Matrix4<f32>) -> bool {
		if uniform == -1 {return false}
		unsafe {
			self.gl.UniformMatrix4fv(uniform, 1, gl::FALSE, matrix.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_matrix4_raw(&self, uniform: UniformLocation, matrix: &[f32]) -> bool {
		if uniform == -1 {return false}
		if matrix.len() != 16 {return false}
		unsafe {
			self.gl.UniformMatrix4fv(uniform, 1, gl::FALSE, matrix.as_ptr());
			true
		}
	}
	
	#[allow(dead_code)]
	pub fn set_uniform_matrix4_raw_transposed(&self, uniform: UniformLocation, matrix: &[f32]) -> bool {
		if uniform == -1 {return false}
		if matrix.len() != 16 {return false}
		unsafe {
			self.gl.UniformMatrix4fv(uniform, 1, gl::TRUE, matrix.as_ptr());
			true
		}
	}
}
use super::Gl;
use super::BufferObjectRef;

#[allow(dead_code)]
pub struct VertexArrayBuilder {
	gl: Gl,
	id: gl::types::GLuint,
	buffers: smallvec::SmallVec<[BufferObjectRef;2]>,
}

impl VertexArrayBuilder {
	pub fn new(gl: &Gl) -> Self {
		trace!("Allocating vertex array...");
		let mut id: gl::types::GLuint = 0;
		
		unsafe {
			gl.GenVertexArrays(1, &mut id);
			gl.BindVertexArray(id);
		}
		
		Self {
			gl: gl.clone(),
			id,
			buffers: smallvec![],
		}
	}
	
	pub fn attach_buffer(mut self, buffer_object: BufferObjectRef, attributes: &[VertexArrayAttrib], label: Option<&str>) -> Self {
		trace!("Attaching buffer to vertex array with {} attributes...", attributes.len());
		
		if let Some(label) = label {
			self.gl.label_object(
				gl::BUFFER,
				buffer_object.id,
				&format!("{} Buffer", label)
			);
		}
		
		unsafe {
			self.gl.BindBuffer(buffer_object.target, buffer_object.id);
			
			if attributes.len() > 0 {
				for attrib in attributes {
					attrib.apply(&self.gl);
				}
			}
		}
		
		
		self.buffers.push(buffer_object);
		
		self
	}
	
	pub fn set_label(self, label: &str) -> Self {
		self.gl.label_object(
			gl::VERTEX_ARRAY,
			self.id,
			&format!("{} Descriptor", label)
		);
		self
	}
	
	pub fn build(self, mode: gl::types::GLenum, elements: usize) -> VertexArray {
		unsafe {
			self.gl.BindVertexArray(0);
		}
		
		VertexArray {
			gl: self.gl.clone(),
			id: self.id,
			mode,
			count: elements as gl::types::GLsizei,
			buffers: self.buffers,
		}
	}
}




#[allow(dead_code)]
pub struct VertexArray {
	pub gl: Gl,
	pub id: gl::types::GLuint,
	pub mode: gl::types::GLenum,
	pub count: gl::types::GLsizei,
	pub buffers: smallvec::SmallVec<[BufferObjectRef;2]>,
}

impl VertexArray {
	#[allow(dead_code)]
	pub fn draw_arrays(&self, gl: &Gl) {
		unsafe {
			gl.BindVertexArray(self.id);
			gl.DrawArrays(self.mode, 0, self.count);
		}
	}
	
	#[allow(dead_code)]
	pub fn draw_arrays_dynamic(&self, gl: &Gl, count: usize) {
		let count = count as gl::types::GLsizei;
		unsafe {
			gl.BindVertexArray(self.id);
			gl.DrawArrays(self.mode, 0, count);
		}
	}
	
	#[allow(dead_code)]
	pub fn draw_elements(&self, gl: &Gl, type_: gl::types::GLenum) {
		let count = self.count as gl::types::GLsizei;
		unsafe {
			gl.BindVertexArray(self.id);
			gl.DrawElements(self.mode, count, type_, 0 as *const gl::types::GLvoid);
		}
	}
	
	#[allow(dead_code)]
	pub fn draw_elements_dynamic(&self, gl: &Gl, count: usize, type_: gl::types::GLenum) {
		let count = count as gl::types::GLsizei;
		unsafe {
			gl.BindVertexArray(self.id);
			gl.DrawElements(self.mode, count, type_, 0 as *const gl::types::GLvoid);
		}
	}
}

impl Drop for VertexArray {
	fn drop(&mut self) {
		unsafe {
			let tmp: smallvec::SmallVec<[gl::types::GLuint;2]> = self.buffers.iter().map(|item| item.id).collect();
			self.gl.DeleteBuffers(tmp.len() as gl::types::GLsizei, tmp.as_ptr());
			
			let tmp = [self.id];
			self.gl.DeleteVertexArrays(1, tmp.as_ptr());
		}
	}
}




#[allow(dead_code)]
pub struct VertexArrayAttrib {
	index: gl::types::GLuint,
	size: gl::types::GLint,
	type_: gl::types::GLenum,
	normalized: gl::types::GLboolean,
	stride: gl::types::GLsizei,
	pointer: *const gl::types::GLvoid,
}

impl VertexArrayAttrib {
	#[allow(dead_code)]
	pub fn from_bytes(
		index: usize,
		size: usize,
		type_: gl::types::GLenum,
		normalized: bool,
		stride: usize,
		pointer: usize,
	) -> Self {
		Self {
			index: index as gl::types::GLuint,
			size: size as gl::types::GLint,
			type_,
			normalized: if normalized {gl::TRUE} else {gl::FALSE},
			stride: stride as gl::types::GLsizei,
			pointer: pointer as *const gl::types::GLvoid,
		}
	}
	
	#[allow(dead_code)]
	pub fn from_type<Element>(
		index: usize,
		size: usize,
		type_: gl::types::GLenum,
		normalized: bool,
		stride: usize,
		pointer: usize,
	) -> Self {
		let esize = std::mem::size_of::<Element>();
		Self {
			index: index as gl::types::GLuint,
			size: size as gl::types::GLint,
			type_,
			normalized: if normalized {gl::TRUE} else {gl::FALSE},
			stride: (stride*esize) as gl::types::GLsizei,
			pointer: (pointer*esize) as *const gl::types::GLvoid,
		}
	}
	
	pub unsafe fn apply(&self, gl: &Gl) {
		gl.EnableVertexAttribArray(self.index);
		gl.VertexAttribPointer(
			self.index,
			self.size,
			self.type_,
			self.normalized,
			self.stride,
			self.pointer
		);
	}
}

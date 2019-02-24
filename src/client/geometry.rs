extern crate cgmath;
extern crate gl;

pub struct SimpleVAO {
	handle: gl::types::GLuint,
	count: i32,
}

impl SimpleVAO {
	pub fn draw(&self, mode: u32) {
		unsafe {
			gl::BindVertexArray(self.handle);
			gl::DrawArrays(mode, 0, self.count);
		}
	}
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_planequad(s: f32) -> SimpleVAO {
	let vertices: Vec<f32> = vec![
		-s, 0.0,  s,
		s, 0.0,  s,
		-s, 0.0, -s,
		s, 0.0,  s,
		s, 0.0, -s,
		-s, 0.0, -s
	];
	
	let mut vbo: gl::types::GLuint = 0;
	
	unsafe {
		gl::GenBuffers(1, &mut vbo);
	}
	
	unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
			vertices.as_ptr() as *const gl::types::GLvoid,
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
			0,
			3,
			gl::FLOAT, gl::FALSE,
			(3 * std::mem::size_of::<f32>()) as gl::types::GLint,
			std::ptr::null()
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}
	
	SimpleVAO {
		handle: vao,
		count: (vertices.len()/3) as i32
	}
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_grid() -> SimpleVAO {
	let mut vertices: Vec<f32> = vec![];
	
	let range: i32 = 256;
	let size: f32 = range as f32;
	
	for x in -range .. range {
		vertices.extend(&vec![
			-size, 0.0, x as f32,
			size, 0.0, x as f32
		]);
		vertices.extend(&vec![
			x as f32, 0.0, -size,
			x as f32, 0.0, size
		]);
	}
	
	let mut vbo: gl::types::GLuint = 0;
	
	unsafe {
		gl::GenBuffers(1, &mut vbo);
	}
	
	unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
			vertices.as_ptr() as *const gl::types::GLvoid,
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
			0,
			3,
			gl::FLOAT, gl::FALSE,
			(3 * std::mem::size_of::<f32>()) as gl::types::GLint,
			std::ptr::null()
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}
	
	SimpleVAO {
		handle: vao,
		count: (vertices.len()/2) as i32
	}
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_test() -> SimpleVAO {
	let mut vertices: Vec<f32> = vec![
		-0.5, -0.5, -10.0,
		0.5, -0.5, -10.0,
		0.0, 0.5, -10.0
	];
	
	vertices.extend(&vec![
		-20.0, 0.0, -20.0,
		0.0, 0.0,  20.0,
		20.0, 0.0, -20.0
	]);
	
	vertices.extend(&vec![
		-5.0, 0.0, 30.0,
		0.0, 9.0, 30.0,
		5.0, 0.0, 30.0
	]);
	
	let mut vbo: gl::types::GLuint = 0;
	unsafe {
		gl::GenBuffers(1, &mut vbo);
	}
	unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
			vertices.as_ptr() as *const gl::types::GLvoid,
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
			0,
			3,
			gl::FLOAT, gl::FALSE,
			(3 * std::mem::size_of::<f32>()) as gl::types::GLint,
			std::ptr::null()
		);
		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}
	
	SimpleVAO {
		handle: vao,
		count: (vertices.len()/3) as i32
	}
}

/*
	This file contains functions to generate geometry of various kinds.
	it also contains the SimpleVAO-struct for easy rendering.
*/

extern crate cgmath;
extern crate gl;

pub struct SimpleVao {
	handle: gl::types::GLuint,
	count: i32,
}

impl SimpleVao {
	pub fn draw(&self, mode: u32) {
		unsafe {
			gl::BindVertexArray(self.handle);
			gl::DrawArrays(mode, 0, self.count);
		}
	}
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_cube(s: f32) -> SimpleVao {
	let mut builder = SimpleVaoBuilder::new();
	
	builder.push_quads(vec![ // top
		-s, s,  s, // a
		 s, s,  s, // b
		 s, s, -s, // c
		-s, s, -s, // d
	]);
	
	builder.push_quads(vec![ // bottom
		-s, -s, -s, // d
		 s, -s, -s, // c
		 s, -s,  s, // b
		-s, -s,  s, // a
	]);
	
	builder.push_quads(vec![ // front
	    -s,  s, -s, // a
	     s,  s, -s, // b
	     s, -s, -s, // c
	    -s, -s, -s, // d
	]);
	
	builder.push_quads(vec![ // back
	    -s, -s, s, // d
	     s, -s, s, // c
	     s,  s, s, // b
	    -s,  s, s, // a
	]);
	
	builder.push_quads(vec![ // left
	    -s,  s,  s, // a
	    -s,  s, -s, // b
	    -s, -s, -s, // c
	    -s, -s,  s, // d
	]);
	
	builder.push_quads(vec![ // right
	    s, -s,  s, // d
	    s, -s, -s, // c
	    s,  s, -s, // b
	    s,  s,  s, // a
	]);
	
	builder.build()
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_planequad(s: f32) -> SimpleVao {
	let mut builder = SimpleVaoBuilder::new();
	builder.push_quads(vec![
		-s, 0.0,  s,
		s, 0.0,  s,
		s, 0.0, -s,
		-s, 0.0, -s
	]);
	builder.build()
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_grid() -> SimpleVao {
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
	
	SimpleVao {
		handle: vao,
		count: (vertices.len()/2) as i32
	}
}

////////////////////////////////////////////////////////////////////////////////

pub fn geometry_test() -> SimpleVao {
	let mut builder = SimpleVaoBuilder::new();
	
	builder.push_vertices(vec![
		-0.5, -0.5, -10.0,
		0.5, -0.5, -10.0,
		0.0, 0.5, -10.0
	]);
	
	builder.push_vertices(vec![
		-20.0, 0.0, -20.0,
		0.0, 0.0,  20.0,
		20.0, 0.0, -20.0
	]);
	
	builder.push_vertices(vec![
		-5.0, 0.0, 30.0,
		0.0, 9.0, 30.0,
		5.0, 0.0, 30.0
	]);
	
	builder.build()
}

////////////////////////////////////////////////////////////////////////////////

struct SimpleVaoBuilder {
	vertices: Vec<f32>,
	texcoord: Vec<f32>,
}

impl SimpleVaoBuilder {
	
	pub fn new() -> SimpleVaoBuilder {
		SimpleVaoBuilder {
			vertices: vec![],
			texcoord: vec![],
		}
	}
	
	pub fn push_vertex(&mut self, x: f32, y: f32, z: f32) {
		self.vertices.push(x);
		self.vertices.push(y);
		self.vertices.push(z);
		self.vertices.push(0.0);
		self.vertices.push(0.0);
	}
	
	pub fn push_vertex_with_uv(&mut self, x: f32, y: f32, z: f32, u: f32, v: f32) {
		self.vertices.push(x);
		self.vertices.push(y);
		self.vertices.push(z);
		self.vertices.push(u);
		self.vertices.push(v);
	}
	
	pub fn push_vertices(&mut self, other: Vec<f32>) {
		if (other.len() % 3) != 0 {
			panic!("Attempted to push non-trinary vertex.");
		}
		
		let num = other.len() / 3;
		for i in 0..num {
			self.push_vertex(
				other[i*3+0],
				other[i*3+1],
				other[i*3+2]
			);
		}
	}
	
	pub fn push_quads(&mut self, quad: Vec<f32>) {
		if (quad.len() % 3*4) != 0 {
			panic!("Attempted to push non-quadliteral quads.");
		}
		
		/* quad:
		- A = 0 1 2
		- B = 3 4 5
		- C = 6 7 8
		- D = 9 10 11
		*/
		self.push_vertex_with_uv(quad[0], quad[1],  quad[2],  0.0, 0.0); // A
		self.push_vertex_with_uv(quad[3], quad[4],  quad[5],  1.0, 0.0); // B
		self.push_vertex_with_uv(quad[9], quad[10], quad[11], 0.0, 1.0); // D
		self.push_vertex_with_uv(quad[3], quad[4],  quad[5],  1.0, 0.0); // B
		self.push_vertex_with_uv(quad[6], quad[7],  quad[8],  1.0, 1.0); // C
		self.push_vertex_with_uv(quad[9], quad[10], quad[11], 0.0, 1.0); // D
	}
	
	pub fn build(&self) -> SimpleVao {
		let mut vbo_vertex: gl::types::GLuint = 0;
		unsafe {
			println!("Allocating vertex buffer for geometry...");
			gl::GenBuffers(1, &mut vbo_vertex);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo_vertex);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
				self.vertices.as_ptr() as *const gl::types::GLvoid,
				gl::STATIC_DRAW
			);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		
		let mut vao: gl::types::GLuint = 0;
		unsafe {
			println!("Allocating vertex array for geometry...");
			gl::GenVertexArrays(1, &mut vao);
			gl::BindVertexArray(vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, vbo_vertex);
			
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0,
				3,
				gl::FLOAT,
				gl::FALSE,
				(5 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
				(0 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
			);
			
			gl::EnableVertexAttribArray(1);
			gl::VertexAttribPointer(
				1,
				2,
				gl::FLOAT,
				gl::FALSE,
				(5 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
				(3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
			);
			
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::BindVertexArray(0);
		}
		
		SimpleVao {
			handle: vao,
			count: (self.vertices.len()/5) as i32
		}
	}
	
}
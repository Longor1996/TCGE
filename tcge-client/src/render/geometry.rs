use super::*;

pub struct SimpleMeshBuilder {
	vertices: smallvec::SmallVec<[f32;32]>
}

impl SimpleMeshBuilder {
	pub fn new() -> SimpleMeshBuilder {
		SimpleMeshBuilder {
			vertices: smallvec![]
		}
	}
	
	#[allow(dead_code)]
	pub fn current(&self) -> usize {
		self.vertices.len() / 5
	}
	
	#[allow(dead_code)]
	pub fn translate_range(&mut self, from: usize, to: Option<usize>, x: f32, y: f32, z: f32) {
		let to = to.unwrap_or(self.vertices.len() / 5);
		
		for vertex in from..to {
			let pos = vertex * 5;
			self.vertices[pos + 0] += x;
			self.vertices[pos + 1] += y;
			self.vertices[pos + 2] += z;
		}
	}
	
	/// Add a new vertex (`x, y, z`) with texture-coordinates `(0.0, 0.0)`.
	#[allow(dead_code)]
	pub fn push_vertex(&mut self, x: f32, y: f32, z: f32) {
		self.vertices.push(x);
		self.vertices.push(y);
		self.vertices.push(z);
		self.vertices.push(0.0);
		self.vertices.push(0.0);
	}
	
	/// Add a new vertex (`x, y, z`) with texture-coordinates `(u, v)`.
	#[allow(dead_code)]
	pub fn push_vertex_with_uv(&mut self, x: f32, y: f32, z: f32, u: f32, v: f32) {
		self.vertices.push(x);
		self.vertices.push(y);
		self.vertices.push(z);
		self.vertices.push(u);
		self.vertices.push(v);
	}
	
	/// Push a large amount of vertices, without texture-coordinates.
	#[allow(dead_code)]
	pub fn push_vertices(&mut self, other: Vec<f32>) {
		if (other.len() % 3) != 0 {
			panic!("Attempted to push non-trinary vertex.");
		}
		
		let num = other.len() / 3;
		for i in 0..num {
			self.push_vertex(
				other[i * 3 + 0],
				other[i * 3 + 1],
				other[i * 3 + 2]
			);
		}
	}
	
	/// Push a quad (4 vertices, two triangles) with texture-coordinates.
	#[allow(dead_code)]
	pub fn push_quads(&mut self, quad: &[f32]) {
		if (quad.len() % 3 * 4) != 0 {
			panic!("Attempted to push non-quadliteral quads.");
		}
		
		/* quad:
		- A = 0 1 2
		- B = 3 4 5
		- C = 6 7 8
		- D = 9 10 11
		*/
		self.push_vertex_with_uv(quad[0], quad[1], quad[2], 0.0, 0.0); // A
		self.push_vertex_with_uv(quad[3], quad[4], quad[5], 1.0, 0.0); // B
		self.push_vertex_with_uv(quad[9], quad[10], quad[11], 0.0, 1.0); // D
		self.push_vertex_with_uv(quad[3], quad[4], quad[5], 1.0, 0.0); // B
		self.push_vertex_with_uv(quad[6], quad[7], quad[8], 1.0, 1.0); // C
		self.push_vertex_with_uv(quad[9], quad[10], quad[11], 0.0, 1.0); // D
	}
	
	/// Uploads the (hopefully valid) mesh to the GPU.
	pub fn build(self, gl: &gl::Gl) -> VertexArray {
		
		let elements = self.vertices.len() / 5;
		
		let vbo = BufferObject::buffer_data(
			gl,
			gl::ARRAY_BUFFER,
			gl::STATIC_DRAW,
			self.vertices.as_slice()
		).to_ref();
		
		let vao = VertexArrayBuilder::new(gl)
			.attach_buffer(vbo, &[VertexArrayAttrib::from_type::<f32>(0, 3, gl::FLOAT, false, 5, 0),
				VertexArrayAttrib::from_type::<f32>(1, 2, gl::FLOAT, false, 5, 3)], None)
			.build(
				gl::TRIANGLES,
				elements
			);
		vao
	}
}

#[allow(dead_code)]
pub fn geometry_plane(gl: &gl::Gl, s: f32) -> VertexArray {
	let mut builder = SimpleMeshBuilder::new();
	builder.push_quads(&[
		-s, 0.0,  s,
		s, 0.0,  s,
		s, 0.0, -s,
		-s, 0.0, -s
	]);
	builder.build(gl)
}

#[allow(dead_code)]
pub fn geometry_plane_subdivided(gl: &gl::Gl, s: f32, d: i32) -> VertexArray {
	let mut builder = SimpleMeshBuilder::new();
	
	let sd = s / d as f32;
	let sdh = sd / 2.0;
	
	for z in -d..d {
		for x in -d..d {
			let cz = z as f32 * sd;
			let cx = x as f32 * sd;
			
			builder.push_quads(&[
				cx-sdh, 0.0, cz+sdh,
				cx+sdh, 0.0, cz+sdh,
				cx+sdh, 0.0, cz-sdh,
				cx-sdh, 0.0, cz-sdh
			]);
		}
	}
	
	builder.build(gl)
}

////////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub fn geometry_cube(gl: &gl::Gl, s: f32) -> VertexArray {
	let mut builder = SimpleMeshBuilder::new();
	
	builder.push_quads(&[ // top
		-s, s,  s, // a
		s, s,  s, // b
		s, s, -s, // c
		-s, s, -s, // d
	]);
	
	builder.push_quads(&[ // bottom
		-s, -s, -s, // d
		s, -s, -s, // c
		s, -s,  s, // b
		-s, -s,  s, // a
	]);
	
	builder.push_quads(&[ // front
		-s,  s, -s, // a
		s,  s, -s, // b
		s, -s, -s, // c
		-s, -s, -s, // d
	]);
	
	builder.push_quads(&[ // back
		-s, -s, s, // d
		s, -s, s, // c
		s,  s, s, // b
		-s,  s, s, // a
	]);
	
	builder.push_quads(&[ // left
		-s,  s,  s, // a
		-s,  s, -s, // b
		-s, -s, -s, // c
		-s, -s,  s, // d
	]);
	
	builder.push_quads(&[ // right
		s, -s,  s, // d
		s, -s, -s, // c
		s,  s, -s, // b
		s,  s,  s, // a
	]);
	
	builder.build(gl)
}
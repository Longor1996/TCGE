use super::super::super::resources;
use super::super::geometry;
use super::cgmath::SquareMatrix;
use super::utility;
use std::io::{BufRead, BufReader};

const FONT_DATA_PNG: &str = "fonts/ascii/Hack-Regular.ttf.sdf.png";
const FONT_DATA_TXT: &str = "fonts/ascii/Hack-Regular.ttf.sdf.txt";
const FONT_MATERIAL: &str = "shaders/sdf-text";

pub struct AsciiTextRenderer {
	material: AsciiTextRendererMaterial,
	characters: Vec<AsciiTextRendererChar>,
	quad: geometry::SimpleVao,
	buffer: Vec<f32>,
	buffer_vao: gl::types::GLuint,
	buffer_vbo: gl::types::GLuint,
	buffer_size: gl::types::GLsizeiptr,
	pub transform: cgmath::Matrix4<f32>,
}

impl AsciiTextRenderer {
	
	pub fn load(res: &resources::Resources) -> Result<AsciiTextRenderer, utility::Error> {
		let material = AsciiTextRendererMaterial::new(res)?;
		
		let gpu = AsciiTextRenderer::prepare_gpu_objects();
		
		let file = res.open_stream(FONT_DATA_TXT)
			.map_err(|e| utility::Error::ResourceLoad { name: FONT_DATA_TXT.to_string(), inner: e })?;
		
		// Allocate character-table and fill it with 'null'
		let mut chars: Vec<AsciiTextRendererChar> = Vec::with_capacity(256);
		for x in 0 .. 256 {
			chars.push(AsciiTextRendererChar::from_nothing(x));
		}
		
		for line in BufReader::new(file).lines() {
			let line = line.expect("Error while reading font definition.");
			
			if ! line.starts_with("char ") {
				continue;
			}
			
			let mut char = AsciiTextRendererChar::from_line(&line[5..]);
			
			char.uv = material.sdfmap.get_uv_rect(
				char.x, char.y,
				char.width, char.height
			);
			
			chars[char.id] = char;
		}
		
		Ok(AsciiTextRenderer {
			material: material,
			characters: chars,
			transform: cgmath::Matrix4::identity(),
			quad: geometry::geometry_planequad(256.0),
			buffer: vec![],
			buffer_vbo: gpu.0,
			buffer_vao: gpu.1,
			buffer_size: gpu.2,
		})
	}
	
	pub fn prepare_gpu_objects() -> (gl::types::GLuint, gl::types::GLuint, gl::types::GLsizeiptr) {
		let buffer_size = (1024*1024) * std::mem::size_of::<f32>() as gl::types::GLsizeiptr;
		let mut buffer_vbo: gl::types::GLuint = 0;
		unsafe {
			gl::GenBuffers(1, &mut buffer_vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, buffer_vbo);
			gl::BufferData(
				gl::ARRAY_BUFFER,
				buffer_size as gl::types::GLsizeiptr,
				(0) as *const gl::types::GLvoid,
				gl::DYNAMIC_STORAGE_BIT
			);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		
		let mut buffer_vao: gl::types::GLuint = 0;
		unsafe {
			gl::GenVertexArrays(1, &mut buffer_vao);
			gl::BindVertexArray(buffer_vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, buffer_vbo);
			
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0,
				2,
				gl::FLOAT,
				gl::FALSE,
				(4 * std::mem::size_of::<f32>()) as gl::types::GLint,
				(0 * std::mem::size_of::<f32>()) as *const std::ffi::c_void
			);
			
			gl::EnableVertexAttribArray(1);
			gl::VertexAttribPointer(
				1,
				2,
				gl::FLOAT,
				gl::FALSE,
				(4 * std::mem::size_of::<f32>()) as gl::types::GLint,
				(2 * std::mem::size_of::<f32>()) as *const std::ffi::c_void
			);
			
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::BindVertexArray(0);
		}
		
		return (buffer_vbo, buffer_vao, buffer_size);
	}
	
	pub fn draw_text(&mut self, text: String, x: f32, y: f32) {
		
		let position = cgmath::Vector3::<f32> {x, y, z: 0.0};
		let transform = self.transform
			* cgmath::Matrix4::from_translation(position)
			* cgmath::Matrix4::from_angle_x(cgmath::Deg(90.0));
		let color = cgmath::Vector4::<f32> {x: 1.0, y: 1.0, z: 1.0, w: 1.0};
		
		self.material.shader.set_used();
		self.material.shader.uniform_matrix4(self.material.uniform_matrix, transform);
		self.material.shader.uniform_vector4(self.material.uniform_color, color);
		self.material.shader.uniform_sampler(self.material.uniform_sdfmap, 0);
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.material.sdfmap.id);
		}
		
		// TODO: Fill vertex data into the buffer, then copy to GPU
		self.buffer.clear();
		
		unsafe {
			let buflen_cpu = self.buffer.len();
			let buflen_gpu = self.buffer_size as usize / std::mem::size_of::<f32>();
			if buflen_cpu > buflen_gpu {
				eprintln!("Text Geometry Buffer Overflow: {} > {}", buflen_cpu, buflen_gpu);
				return;
			}
			
			gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_vbo);
			gl::BufferSubData(
				gl::ARRAY_BUFFER, 0,
				(buflen_cpu / std::mem::size_of::<f32>()) as isize,
			    self.buffer.as_ptr() as *const gl::types::GLvoid
			);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		
		unsafe {
			let triangles_count = (self.buffer.len() / 3) as i32;
			if triangles_count != 0 {
				gl::BindVertexArray(self.buffer_vao);
				gl::DrawArrays(gl::TRIANGLES, 0, triangles_count);
				gl::BindVertexArray(0);
			}
		}
	}
	
}

#[derive(Clone,Copy)]
struct AsciiTextRendererChar {
	id: usize,
	x: u32,
	y: u32,
	width: u32,
	height: u32,
	xoffset: f32,
	yoffset: f32,
	xadvance: f32,
	uv: [f32;4],
}

impl AsciiTextRendererChar {
	pub fn from_nothing(id: usize) -> AsciiTextRendererChar {
		AsciiTextRendererChar {
			id,
			x: 0, y: 0,
			width: 0, height: 0,
			xoffset: 0.0, yoffset: 0.0,
			xadvance: 0.0,
			uv: [0.0, 0.0, 0.0, 0.0]
		}
	}
	
	pub fn from_line(line: &str) -> AsciiTextRendererChar {
		let attribs: Vec<&str> = line.split_whitespace().collect();
		let mut char = AsciiTextRendererChar::from_nothing(0);
		
		for attribute in attribs {
			let splitpos = attribute.find("=").expect("Invalid char definition.");
			
			let key_val = attribute.split_at(splitpos);
			let key = key_val.0;
			let value = &(key_val.1)[1..];
			
			match key {
				"id" => char.id = value.parse::<usize>().expect("Could not parse 'id'"),
				"x" => char.x = value.parse::<u32>().expect("Could not parse 'x'"),
				"y" => char.y = value.parse::<u32>().expect("Could not parse 'y'"),
				"width" => char.width = value.parse::<u32>().expect("Could not parse 'width'"),
				"height" => char.height = value.parse::<u32>().expect("Could not parse 'height'"),
				"xoffset" => char.xoffset = value.parse::<f32>().expect("Could not parse 'xoffset'"),
				"yoffset" => char.yoffset = value.parse::<f32>().expect("Could not parse 'yoffset'"),
				"xadvance" => char.xadvance = value.parse::<f32>().expect("Could not parse 'xadvance'"),
				_ => {}
			}
		}
		
		return char;
	}
}

pub struct AsciiTextRendererMaterial {
	pub shader: utility::Program,
	pub sdfmap: utility::Texture,
	pub uniform_matrix: i32,
	pub uniform_sdfmap: i32,
	pub uniform_color: i32,
}

impl AsciiTextRendererMaterial {
	pub fn new(res: &resources::Resources) -> Result<AsciiTextRendererMaterial, utility::Error> {
		let sdfmap = utility::Texture::from_res(&res, FONT_DATA_PNG)?;
		let shader = utility::Program::from_res(&res, FONT_MATERIAL)?;
		
		let uniform_matrix = shader.uniform_location("transform");
		let uniform_sdfmap = shader.uniform_location("sdfmap");
		let uniform_color = shader.uniform_location("color");
		
		Ok(AsciiTextRendererMaterial {shader, sdfmap,
			uniform_matrix,
			uniform_sdfmap,
			uniform_color
		})
	}
}

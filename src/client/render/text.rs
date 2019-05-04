//! Module for rendering text in the extended ASCII-range, by using signed-distance fields.

use super::super::super::resources;
use super::cgmath::SquareMatrix;
use super::utility;
use std::io::{BufRead, BufReader};

const FONT_DATA_PNG: &str = "fonts/hack/hack.png";
const FONT_DATA_TXT: &str = "fonts/hack/hack.fnt";
const FONT_MATERIAL: &str = "shaders/sdf-text";

pub struct AsciiTextRenderer {
	material: AsciiTextRendererMaterial,
	characters: Vec<AsciiTextRendererChar>,
	scale: f32,
	buffer: Vec<f32>,
	buffer_vao: gl::types::GLuint,
	buffer_vbo: gl::types::GLuint,
	buffer_size: gl::types::GLsizeiptr,
	pub transform: cgmath::Matrix4<f32>,
}

impl AsciiTextRenderer {
	
	pub fn load(res: &resources::Resources) -> Result<AsciiTextRenderer, utility::Error> {
		info!("Loading font: {}, {}", FONT_DATA_TXT, FONT_DATA_PNG);
		let material = AsciiTextRendererMaterial::new(res)?;
		
		debug!("Preparing GPU resources...");
		let gpu = AsciiTextRenderer::prepare_gpu_objects(&material);
		
		let mut buffer = vec![];
		buffer.resize(gpu.2 as usize / std::mem::size_of::<f32>(), 0.0);
		
		debug!("Loading font: {}", FONT_DATA_TXT);
		let file = res.open_stream(FONT_DATA_TXT)
			.map_err(|e| utility::Error::ResourceLoad { name: FONT_DATA_TXT.to_string(), inner: e })?;
		
		// Allocate character-table and fill it with 'null'
		let capacity = 8400;
		let mut chars: Vec<AsciiTextRendererChar> = Vec::with_capacity(capacity + 1);
		for x in 0 .. capacity {
			chars.push(AsciiTextRendererChar::from_nothing(x));
		}
		
		let mut scale = 32.0; // default
		
		debug!("Parsing font: {}", FONT_DATA_TXT);
		for line in BufReader::new(file).lines() {
			let line = line.expect("Error while reading font definition.");
			
			if line.starts_with("info ") {
				AsciiTextRenderer::parse_line(&line, &mut |key, value| {
					if key == "size" {
						scale = value.parse::<f32>()
							.map_err(|e| utility::Error::ValueParse { name: e.to_string() })?;
					}
					
					Ok(())
				})?;
			}
			
			if line.starts_with("char ") {
				let mut char = AsciiTextRendererChar::from_nothing(0);
				AsciiTextRenderer::parse_line(&line, &mut |key, value| {
					match key {
						"id" => char.id = value.parse::<usize>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'id'"))?,
						
						"x" => char.x = value.parse::<u32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'x'"))?,
						
						"y" => char.y = value.parse::<u32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'y'"))?,
						
						"width" => char.width = value.parse::<u32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'width'"))?,
						
						"height" => char.height = value.parse::<u32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'height'"))?,
						
						"xoffset" => char.xoffset = value.parse::<f32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'xoffset'"))?,
						
						"yoffset" => char.yoffset = value.parse::<f32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'yoffset'"))?,
						
						"xadvance" => char.xadvance = value.parse::<f32>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'xadvance'"))?,
						
						"page" => char.page = value.parse::<usize>()
							.map_err(|_e| utility::Error::new_valerr("Could not parse 'page'"))?,
						
						_ => {}
					}
					
					Ok(())
				})?;
				
				char.uv = material.sdfmap.get_uv_rect(
					char.x, char.y,
					char.width, char.height
				);
				
				chars[char.id] = char;
				continue;
			}
			
			// unknown line!
			continue;
		}
		
		Ok(AsciiTextRenderer {
			material,
			characters: chars,
			transform: cgmath::Matrix4::identity(),
			scale,
			buffer,
			buffer_vbo: gpu.0,
			buffer_vao: gpu.1,
			buffer_size: gpu.2,
		})
	}
	
	fn parse_line<'a>(
		line: &'a String,
		callback: &mut FnMut(&str, &str) -> Result<(), utility::Error>
	) -> Result<(), utility::Error> {
		let mut chars = line.chars()
			.chain(" ".chars())
			.enumerate()
			.peekable();
		
		let mut state = 0;
		let mut key_start = 0;
		let mut key_end = 0;
		let mut val_start = 0;
		loop {
			let (pos, current) = match chars.peek() {
				Some(x) => *x,
				None => return Ok(())
			};
			
			// This is an extremely simple state-machine.
			match state {
				0 => { // SEEKING
					if current.is_whitespace() {
						chars.next();
						continue;
					}
					if current.is_alphabetic() {
						key_start = pos;
						state = 1;
						continue;
					}
				},
				1 => {// READING KEY
					key_end = pos;
					if current.is_whitespace() {
						callback(&line[key_start .. key_end], "")?;
						chars.next(); // consume whitespace
						state = 0;
						continue;
					}
					else if current == '=' {
						chars.next();  // consume equal-sign
						state = 2;
						continue;
					}
					else {
						chars.next(); // consume key char
					}
				},
				2 => {// VALUE QUOTING CHECK
					if current == '"' {
						chars.next(); // consume quote
						val_start = pos + 1;
						state = 4;
						continue;
					}
					else {
						val_start = pos;
						state = 3;
						continue;
					}
				},
				3 => { // READING UNQUOTED VALUE
					if current.is_whitespace() {
						callback(
							&line[key_start .. key_end],
							&line[val_start .. pos]
						)?;
						chars.next(); // consume whitespace
						state = 0;
						continue;
					}
					else {
						chars.next(); // consume value char
					}
				},
				4 => { // READING QUOTED VALUE
					if current == '"' {
						callback(
							&line[key_start .. key_end],
							&line[val_start .. pos]
						)?;
						chars.next(); // consume quote
						state = 0;
						continue;
					}
					else {
						chars.next(); // consume value char
					}
				},
				_ => panic!("Invalid state!")
			}
		}
	}
	
	pub fn prepare_gpu_objects(
		_material: &AsciiTextRendererMaterial
	) -> (
		gl::types::GLuint,
		gl::types::GLuint,
		gl::types::GLsizeiptr
	) {
		let buffer_size = (1024*1024) * std::mem::size_of::<f32>() as gl::types::GLsizeiptr;
		let mut buffer_vbo: gl::types::GLuint = 0;
		unsafe {
			trace!("Allocating text geometry buffer...");
			gl::GenBuffers(1, &mut buffer_vbo);
			gl::BindBuffer(gl::ARRAY_BUFFER, buffer_vbo);
			gl::BufferStorage(
				gl::ARRAY_BUFFER,
				buffer_size as gl::types::GLsizeiptr,
				std::ptr::null(),
				gl::MAP_WRITE_BIT
			);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		
		let mut buffer_vao: gl::types::GLuint = 0;
		unsafe {
			trace!("Allocating text geometry descriptor...");
			gl::GenVertexArrays(1, &mut buffer_vao);
			gl::BindVertexArray(buffer_vao);
			gl::BindBuffer(gl::ARRAY_BUFFER, buffer_vbo);
			
			gl::EnableVertexAttribArray(0);
			gl::VertexAttribPointer(
				0,
				2,
				gl::FLOAT,
				gl::FALSE,
				(4 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
				(0 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
			);
			
			gl::EnableVertexAttribArray(1);
			gl::VertexAttribPointer(
				1,
				2,
				gl::FLOAT,
				gl::FALSE,
				(4 * std::mem::size_of::<f32>()) as gl::types::GLsizei,
				(2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
			);
			
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
			gl::BindVertexArray(0);
		}
		
		utility::gl_label_object(
			gl::VERTEX_ARRAY,
			buffer_vao,
			"Dynamic Text Renderer Descriptor"
		);
		
		utility::gl_label_object(
			gl::BUFFER,
			buffer_vbo,
			"Dynamic Text Renderer Geometry"
		);
		
		trace!("Allocated text geometry buffers.");
		return (buffer_vbo, buffer_vao, buffer_size);
	}
	
	pub fn draw_text(&mut self, text: String, font_size: f32, x: f32, y: f32) {
		
		let color = cgmath::Vector4::<f32> {x: 1.0, y: 1.0, z: 1.0, w: 1.0};
		
		self.material.shader.set_used();
		self.material.shader.uniform_matrix4(self.material.uniform_matrix, self.transform);
		self.material.shader.uniform_vector4(self.material.uniform_color, color);
		self.material.shader.uniform_scalar(self.material.uniform_spread, 8.0);
		self.material.shader.uniform_scalar(self.material.uniform_scale, font_size / self.scale);
		self.material.shader.uniform_sampler(self.material.uniform_sdfmap, 0);
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, self.material.sdfmap.id);
		}
		
		self.buffer.clear();
		let xstart = x;
		let mut xpos = x;
		let mut ypos = y;
		
		for char in text.chars() {
			if char == '\n' {
				xpos = xstart;
				ypos += font_size;
				continue;
			}
			
			self.draw_char(
				&mut xpos,
				&mut ypos,
				font_size,
				char
			);
		}
		
		unsafe {
			let buflen_cpu = self.buffer.len(); // individual float elements
			let buflen_gpu = (self.buffer_size as usize) / 4;
			if buflen_cpu > buflen_gpu {
				warn!("Text Geometry Buffer Overflow: {} > {}", buflen_cpu, buflen_gpu);
				return;
			}
			
			gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_vbo);
			let hndl = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut f32;
			if hndl.is_null() {
				panic!("OpenGL returned NIL-handle.");
			}
			let len = (self.buffer.len() * std::mem::size_of::<f32>()) as usize;
			self.buffer.as_ptr().copy_to(hndl, len);
			gl::UnmapBuffer(gl::ARRAY_BUFFER);
			gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		}
		
		unsafe {
			let vertices = (self.buffer.len() / 4) as i32;
			
			if vertices != 0 {
				gl::BindVertexArray(self.buffer_vao);
				gl::DrawArrays(gl::TRIANGLES, 0, vertices);
				gl::BindVertexArray(0);
			}
		}
		
		unsafe {
			gl::BindTexture(gl::TEXTURE_2D, 0);
		}
	}
	
	pub fn draw_char(&mut self, x: &mut f32, y: &mut f32, font_size: f32, character: char) {
		let character = character as usize;
		
		if character >= self.characters.len() {
			return;
		}
		
		let character = &self.characters[character];
		let w  = character.width  as f32 / self.scale*font_size;
		let h  = character.height as f32 / self.scale*font_size;
		let lx = *x + character.xoffset  / self.scale*font_size;
		let ly = *y + character.yoffset  / self.scale*font_size;
		
		let mut temp = vec![
			// triangle top left
			lx + 0.0, ly + 0.0, character.uv[0], character.uv[1],
			lx + (w), ly + 0.0, character.uv[2], character.uv[1],
			lx + 0.0, ly + (h), character.uv[0], character.uv[3],
			
			// triangle bottom right
			lx + (w), ly + 0.0, character.uv[2], character.uv[1],
			lx + (w), ly + (h), character.uv[2], character.uv[3],
			lx + 0.0, ly + (h), character.uv[0], character.uv[3],
		];
		&self.buffer.append(&mut temp);
		
		// increase x position
		*x += character.xadvance / self.scale * font_size;
	}
	
}

#[derive(Clone,Copy)]
struct AsciiTextRendererChar {
	id: usize,
	page: usize,
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
			page: 0,
			x: 0, y: 0,
			width: 0, height: 0,
			xoffset: 0.0, yoffset: 0.0,
			xadvance: 0.0,
			uv: [0.0, 0.0, 0.0, 0.0]
		}
	}
}

pub struct AsciiTextRendererMaterial {
	pub shader: utility::Program,
	pub sdfmap: utility::Texture,
	pub uniform_matrix: i32,
	pub uniform_sdfmap: i32,
	pub uniform_color:  i32,
	pub uniform_spread: i32,
	pub uniform_scale:  i32,
}

impl AsciiTextRendererMaterial {
	pub fn new(res: &resources::Resources) -> Result<AsciiTextRendererMaterial, utility::Error> {
		debug!("Loading font texture...");
		let sdfmap = utility::Texture::from_res(&res, FONT_DATA_PNG, &||{})?;
		
		debug!("Loading font shader...");
		let shader = utility::Program::from_res(&res, FONT_MATERIAL)?;
		
		let uniform_matrix = shader.uniform_location("transform");
		let uniform_sdfmap = shader.uniform_location("sdfmap");
		let uniform_color = shader.uniform_location("color");
		let uniform_spread = shader.uniform_location("spread");
		let uniform_scale = shader.uniform_location("scale");
		
		Ok(AsciiTextRendererMaterial {shader, sdfmap,
			uniform_matrix,
			uniform_sdfmap,
			uniform_color,
			uniform_spread,
			uniform_scale,
		})
	}
}

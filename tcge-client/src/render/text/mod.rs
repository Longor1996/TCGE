use rustc_hash::FxHashMap;
use common::resources::{Resources, ResourceLocation, ResourceError};
use super::{TextureError, VertexArray};

pub type TextRendererComp = crate::WrapperComponent<TextRenderer>;

pub const TEXT_RENDERER_FILES: [(&str, &[u8]); 4] = [
	("core/shaders/text.vert", include_bytes!("sdf-text.vert")),
	("core/shaders/text.frag", include_bytes!("sdf-text.frag")),
	("core/fonts/hack/hack.fnt", include_bytes!("hack/hack.fnt")),
	("core/fonts/hack/hack.png", include_bytes!("hack/hack.png")),
];

mod material;
use material::TextRendererMaterial;

mod parser;
use parser::*;

mod renderer;
use renderer::TextRenderer;

pub fn new(gl: &gl::Gl, res: &Resources, name: &str) -> Result<TextRenderer, TextRendererError>{
	
	gl.push_debug(&format!("Loading font: {}", name));
	
	let font_loc = ResourceLocation::from(format!("core/fonts/{}", name));
	let index_loc = font_loc.add(&format!("/{}.fnt", name));
	
	info!("Loading font: {} -> {}", font_loc, index_loc);
	
	debug!("Preparing GPU resources...");
	let material = TextRendererMaterial::new(gl, res)?;
	let descriptor = prepare_gpu_objects(&gl);
	
	let buffer_size = descriptor.buffers[0].items;
	let buffer = Vec::<f32>::with_capacity(buffer_size);
	
	let characters: FxHashMap<u32, TextGlyph> = FxHashMap::default();
	let metrics = TextMetrics::default();
	let transform = cgmath::One::one();
	
	let mut text = TextRenderer {
		gl: gl.clone(),
		metrics,
		characters,
		material,
		descriptor,
		buffer,
		transform
	};
	
	parse_file(
		res,
		&font_loc,
		&index_loc,
		&mut text,
		name
	)?;
	
	while let Some(error) = gl.get_error() {
		error!("OpenGL error while loading font {}: {}", name, error);
	}
	
	gl.flush();
	gl.pop_debug();
	
	Ok(text)
}

fn prepare_gpu_objects(gl: &gl::Gl) -> super::VertexArray {
	gl.push_debug("Preparing GPU Objects");
	let characters = 1024;
	
	trace!("Allocating vertex buffer...");
	let vertices = super::BufferObject::buffer_storage_empty::<f32>(
		&gl,
		gl::ARRAY_BUFFER,
		gl::MAP_WRITE_BIT,
		characters*4*4
	).to_ref();
	
	let mut indices: Vec<u16> = vec![];
	for i in 0..characters {
		// A: a b d
		// B: b c d
		let o = i as u16 * 4;
		indices.append(&mut vec![
			o+0, o+1, o+3,
			o+1, o+2, o+3,
		]);
	}
	
	trace!("Allocating index buffer...");
	let indices = super::BufferObject::buffer_data(
		&gl,
		gl::ELEMENT_ARRAY_BUFFER,
		gl::DYNAMIC_DRAW,
		&indices
	).to_ref();
	
	trace!("Allocating vertex array...");
	let vao = super::VertexArrayBuilder::new(gl)
		.attach_buffer(
			vertices,
			&vec![
				// Position
				super::VertexArrayAttrib::from_type::<f32>(0, 2, gl::FLOAT, false, 4, 0),
				// TexCoord
				super::VertexArrayAttrib::from_type::<f32>(1, 2, gl::FLOAT, false, 4, 2),
			],
			Some("Dynamic Text Vertices")
		)
		.attach_buffer(
			indices,
			&vec![],
			Some("Dynamic Text Indices")
		)
		.set_label("Dynamic Text")
		.build(gl::TRIANGLES, 0);
	
	while let Some(error) = gl.get_error() {
		error!("OpenGL error while preparing gpu objects for text renderer: {}", error);
	}
	
	unsafe {
		gl.BindVertexArray(0);
		for b in &vao.buffers {
			gl.BindBuffer(b.target, 0);
		}
	}
	
	gl.flush();
	gl.pop_debug();
	
	vao
}

pub struct TextMetrics {
	scale: f32,
	line_height: f32,
	base: f32,
}

impl TextMetrics {
	fn default() -> Self {
		Self {
			scale: 32.0,
			line_height: 38.0,
			base: 30.0,
		}
	}
}

#[derive(Clone,Copy)]
pub struct TextGlyph {
	id: u32,
	codepoint: char,
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

impl TextGlyph {
	pub fn default(id: u32) -> Self {
		Self {
			id,
			codepoint: ' ',
			page: 0,
			x: 0, y: 0,
			width: 0, height: 0,
			xoffset: 0.0, yoffset: 0.0,
			xadvance: 0.0,
			uv: [0.0, 0.0, 0.0, 0.0]
		}
	}
}

pub enum TextRendererError {
	Resource(ResourceError),
	Texture(TextureError),
	Shader(String),
	Parse(String),
}
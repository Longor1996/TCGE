use std::io::{BufRead, BufReader};
use common::resources::{ResourceLocation, Resources, ResourceProvider};
use super::{TextRenderer, TextRendererError, TextGlyph};
use crate::render;

pub fn parse_file(
	res: &Resources,
	font_loc: &ResourceLocation,
	index_loc: &ResourceLocation,
	text: &mut TextRenderer,
	name: &str,
) -> Result<(), TextRendererError> {
	
	let font_file = res.res_as_stream(&index_loc)
		.map_err(|e| TextRendererError::Resource(e))?;
	
	debug!("Parsing font: {}", index_loc);
	for line in BufReader::new(font_file).lines() {
		let line = line.expect("Error while reading font definition.");
		
		if line.starts_with("info ") {
			parse_line(&line, &mut |key, value| {
				if key == "size" {
					text.metrics.scale = value.parse::<f32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?;
				}
				
				Ok(())
			})?;
			continue;
		}
		
		if line.starts_with("page ") {
			let mut page_id: usize = 0;
			let mut page_name = "".to_string();
			parse_line(&line, &mut |key, value| {
				if key == "id" {
					page_id = value.parse::<usize>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?;
				}
				
				if key == "file" {
					page_name = value.to_string();
				}
				
				Ok(())
			})?;
			
			let page_loc = ResourceLocation::from_string(font_loc.inner.clone() + "/" + &page_name);
			
			&text.gl.push_debug(&format!("Loading font-page {}", page_name));
			debug!("Loading font page: {}", page_name);
			let page_tex = render::TextureObjectBuilder::new()
				.name(format!("Font Page {}/{}", name, page_name))
				.build_from_res(&text.gl, &res, &page_loc)
				.map_err(|e| TextRendererError::Texture(e))?;
			
			while let Some(error) = &text.gl.get_error() {
				error!("OpenGL error while loading font-page {}: {}", page_name, error);
			}
			
			&text.gl.flush();
			&text.gl.pop_debug();
			
			text.material.pages.insert(page_id, page_tex);
			continue;
		}
		
		if line.starts_with("common ") {
			parse_line(&line, &mut |key, value| {
				if key == "lineHeight" {
					text.metrics.line_height = value.parse::<f32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?;
				}
				
				if key == "base" {
					text.metrics.base = value.parse::<f32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?;
				}
				
				Ok(())
			})?;
			continue;
		}
		
		if line.starts_with("chars ") {
			parse_line(&line, &mut |key, value| {
				if key == "count" {
					let capacity = value.parse::<usize>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?;
					text.characters.reserve(capacity + 1);
				}
				Ok(())
			})?;
			continue;
		}
		
		if line.starts_with("char ") {
			let mut char = TextGlyph::default(0);
			parse_line(&line, &mut |key, value| {
				match key {
					"id" => {
						char.id = value.parse::<u32>()
							.map_err(|e| TextRendererError::Parse(e.to_string()))?;
						
						char.codepoint = std::char::from_u32(char.id).unwrap();
					},
					
					"x" => char.x = value.parse::<u32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"y" => char.y = value.parse::<u32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"width" => char.width = value.parse::<u32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"height" => char.height = value.parse::<u32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"xoffset" => char.xoffset = value.parse::<f32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"yoffset" => char.yoffset = value.parse::<f32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"xadvance" => char.xadvance = value.parse::<f32>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					"page" => char.page = value.parse::<usize>()
						.map_err(|e| TextRendererError::Parse(e.to_string()))?,
					
					_ => {}
				}
				
				Ok(())
			})?;
			
			char.uv = text.material.pages[&char.page].get_uv_rect(
				char.x, char.y,
				char.width, char.height
			);
			
			text.characters.insert(char.id, char);
			continue;
		}
		
		// unknown line!
		warn!("Unknown command in font-file {}: {}", index_loc, line);
		continue;
	}
	
	Ok(())
}

pub fn parse_line<'a>(
	line: &'a String,
	callback: &mut dyn FnMut(&str, &str) -> Result<(), TextRendererError>
) -> Result<(), TextRendererError> {
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

use super::*;

pub struct TextRenderer {
	pub gl: gl::Gl,
	
	pub metrics: TextMetrics,
	pub characters: FxHashMap<u32, TextGlyph>,
	
	pub material: TextRendererMaterial,
	pub descriptor: super::VertexArray,
	pub buffer: Vec<f32>,
	pub transform: cgmath::Matrix4<f32>,
}

impl TextRenderer {
	
	pub fn draw_text(&mut self, text: &str, font_size: f32, x: f32, y: f32) {
		self.gl.push_debug("Draw Text");
		self.draw_reset(font_size);
		
		self.material.pages[&0].set_used();
		
		let line_start = x;
		let mut x_pos = x;
		let mut y_pos = y;
		let mut page = 0;
		let mut count: usize = 0;
		
		for char in text.chars() {
			if char == '\n' {
				x_pos = line_start;
				y_pos += font_size;
				continue;
			}
			
			let character = match self.characters.get(&(char as u32)) {
				Some(x) => x.clone(),
				None => return
			};
			
			if character.page != page {
				self.draw_submit(count);
				count = 0; // reset counter
				page = character.page;
				self.material.pages[&page].set_used();
			}
			
			if self.draw_char(
				&mut x_pos,
				&mut y_pos,
				font_size,
				&character
			) {
				count += 1;
			}
		}
		
		self.draw_submit(count);
		self.gl.pop_debug();
	}
	
	fn draw_char(&mut self, x: &mut f32, y: &mut f32, font_size: f32, character: &TextGlyph) -> bool {
		let scale = font_size / self.metrics.scale;
		let mut ok = false;
		
		if ! character.codepoint.is_whitespace() {
			let w  = character.width  as f32 * scale;
			let h  = character.height as f32 * scale;
			let lx = *x + character.xoffset  * scale;
			let ly = *y + character.yoffset  * scale;
			
			let mut temp = vec![
				lx + 0.0, ly + 0.0, character.uv[0], character.uv[1],
				lx + (w), ly + 0.0, character.uv[2], character.uv[1],
				lx + (w), ly + (h), character.uv[2], character.uv[3],
				lx + 0.0, ly + (h), character.uv[0], character.uv[3],
			];
			&self.buffer.append(&mut temp);
			ok = true;
		}
		
		// increase x position
		*x += character.xadvance * scale;
		ok
	}
	
	fn draw_reset(&mut self, font_size: f32) {
		let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
		let spread = 8.0;
		let scale = font_size / self.metrics.scale;
		
		self.material.program.set_used();
		self.material.program.set_uniform_vector4_raw(self.material.uniform_color, &color);
		self.material.program.set_uniform_matrix4(self.material.uniform_matrix, &self.transform);
		self.material.program.set_uniform_scalar(self.material.uniform_spread, spread);
		self.material.program.set_uniform_scalar(self.material.uniform_scale, scale);
		self.material.program.set_uniform_sampler(self.material.uniform_sdfmap, 0);
		self.buffer.clear();
	}
	
	fn draw_submit(&mut self, characters: usize) {
		match self.descriptor.buffers[0].buffer_mapped_upload(&self.gl, &self.buffer) {
			Err(e) => error!("{}", e),
			Ok(_) => {
				self.descriptor.draw_elements_dynamic(
					&self.gl,
					characters * 6,
					gl::UNSIGNED_SHORT
				);
			}
		}
		
		// Clear buffer to make space for the next span...
		self.buffer.clear();
	}
	
}

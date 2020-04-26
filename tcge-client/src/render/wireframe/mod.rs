use super::*;
use common::resources::{Resources, ResourceProvider, ResourceLocation, ResourceError};

pub const WIREFRAME_PAINTER_FILES: [(&str, &[u8]); 2] = [
	("core/shaders/wireframe.vert", include_bytes!("wireframe.vert")),
	("core/shaders/wireframe.frag", include_bytes!("wireframe.frag")),
];

pub type WireframePainterComp = crate::WrapperComponent<WireframePainter>;

pub struct WireframePainter {
	pub gl: gl::Gl,
	pub material: WireframeMaterial,
	pub descriptor: super::VertexArray,
	pub buffer: Vec<f32>,
	pub transform: nalgebra_glm::Mat4,
}

impl WireframePainter {
	
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, WireframeError> {
		
		gl.push_debug("Initializing wireframe painter...");
		unsafe {
			gl.LineWidth(2.0);
		}
		
		debug!("Preparing GPU resources...");
		let material = WireframeMaterial::new(gl, res)?;
		let descriptor = WireframePainter::prepare_gpu_objects(&gl);
		
		let buffer_size = descriptor.buffers[0].items;
		let buffer = Vec::<f32>::with_capacity(buffer_size);
		
		let painter = WireframePainter {
			gl: gl.clone(),
			material,
			descriptor,
			buffer,
			transform: nalgebra_glm::translation(&nalgebra_glm::vec3(0.0, 0.0, 0.0))
		};
		
		gl.flush();
		gl.pop_debug();
		
		Ok(painter)
	}
	
	fn prepare_gpu_objects(gl: &gl::Gl) -> super::VertexArray {
		gl.push_debug("Preparing GPU Objects");
		let lines = 1024;
		
		trace!("Allocating vertex buffer...");
		let vertices = super::BufferObject::buffer_storage_empty::<f32>(
			&gl,
			gl::ARRAY_BUFFER,
			gl::MAP_WRITE_BIT,
			lines*2*(3+4)
		).to_ref();
		
		trace!("Allocating vertex array...");
		let vao = super::VertexArrayBuilder::new(gl)
			.attach_buffer(
				vertices,
				&[
					super::VertexArrayAttrib::from_type::<f32>(0, 3, gl::FLOAT, false, 7, 0),
					super::VertexArrayAttrib::from_type::<f32>(1, 4, gl::FLOAT, false, 7, 3)
				],
				Some("Dynamic Line Vertices")
			)
			.set_label("Dynamic Lines")
			.build(gl::LINES, 0);
		
		while let Some(error) = gl.get_error() {
			error!("OpenGL error while preparing gpu objects for wireframe painter: {}", error);
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
	
	fn before_draw(&mut self) {
		self.gl.push_debug("Draw Wireframe");
		self.material.program.set_used();
		self.material.program.set_uniform_matrix4(self.material.uniform_matrix, &self.transform);
		self.buffer.clear();
	}
	
	fn after_draw(&mut self, lines: usize) {
		match self.descriptor.buffers[0].buffer_mapped_upload(&self.gl, &self.buffer) {
			Err(e) => error!("{}", e),
			Ok(_) => {
				self.descriptor.draw_arrays_dynamic(&self.gl, lines * 2);
			}
		}
		self.gl.pop_debug();
		
		// Clear buffer to make space for the next span...
		self.buffer.clear();
	}
}

impl WireframePainter {
	
	pub fn draw_line(&mut self, start: &nalgebra_glm::Vec3, end: &nalgebra_glm::Vec3, color: &nalgebra_glm::Vec4) {
		self.before_draw();
		let lines = 1;
		self.buffer.extend_from_slice(&[
			start.x, start.y, start.z, color.x, color.y, color.z, color.w,
			end.x, end.y, end.z, color.x, color.y, color.z, color.w,
		]);
		self.after_draw(lines);
	}
	
}

pub enum WireframeError {
	Resource(ResourceError),
	Shader(String),
}

pub struct WireframeMaterial {
	pub program: ProgramObject,
	pub uniform_matrix: UniformLocation,
}

impl WireframeMaterial {
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, WireframeError> {
		
		gl.push_debug("Preparing wireframe painter shaders");
		
		let location = ResourceLocation::from_str("core/shaders/wireframe");
		
		debug!("Loading font shader: {}", location);
		
		trace!("Loading vertex shader...");
		let shader_vert = res.res_as_cstring(&location.add(".vert"))
			.map_err(WireframeError::Resource)?;
		
		trace!("Compiling vertex shader...");
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(WireframeError::Shader)?;
		
		trace!("Loading fragment shader...");
		let shader_frag = res.res_as_cstring(&location.add(".frag"))
			.map_err(WireframeError::Resource)?;
		
		trace!("Compiling fragment shader...");
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(WireframeError::Shader)?;
		
		trace!("Linking program...");
		let program = ProgramObject::new(gl, "Wireframe Painter", &smallvec![shader_vert, shader_frag])
			.map_err(WireframeError::Shader)?;
		
		trace!("Fetching uniforms...");
		let uniform_matrix = program.get_uniform_location("transform")
			.map_err(|_| WireframeError::Shader("Missing uniform 'transform'.".to_string()))?;
		
		trace!("Returning material.");
		gl.pop_debug();
		
		Ok(Self {
			program,
			uniform_matrix
		})
	}
}

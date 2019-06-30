use common::resources::*;
use crate::render::*;

pub const SKY_MATERIAL_FILES: [(&str, &[u8]); 2] = [
	("core/shaders/sky.vert", include_bytes!("sky.vert")),
	("core/shaders/sky.frag", include_bytes!("sky.frag")),
];

pub struct SkyRenderer {
	gl: gl::Gl,
	skybox: VertexArray,
	material: SkyMaterial,
}

impl SkyRenderer {
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, SkyMaterialError> {
		let skybox = geometry::geometry_cube(gl, 10.0);
		let shader = SkyMaterial::new(gl, res)?;
		
		Ok(SkyRenderer {
			gl: gl.clone(),
			skybox,
			material: shader,
		})
	}
	
	pub fn render(&mut self, camera: &crate::playground::freecam::Freecam, size: (i32, i32), _now: f64, interpolation: f32, _delta: f32) {
		unsafe {
			self.gl.Disable(gl::DEPTH_TEST);
			self.gl.Disable(gl::CULL_FACE);
		}
		
		let camera_proj = camera.get_gl_projection_matrix(size, interpolation);
		let camera_view = camera.get_gl_view_matrix(false, interpolation);
		let transform = camera_proj * camera_view;
		
		let position = camera.get_position(interpolation);
		let color = [0.3, 0.6, 1.0, 1.0];
		
		self.material.shader.set_used();
		self.material.shader.set_uniform_matrix4(self.material.uniform_matrix, &transform);
		self.material.shader.set_uniform_vector3(self.material.uniform_camera, &position);
		self.material.shader.set_uniform_vector4_raw(self.material.uniform_color, &color);
		
		self.skybox.draw_arrays(&self.gl);
		
		unsafe {
			self.gl.Enable(gl::DEPTH_TEST);
			self.gl.Enable(gl::CULL_FACE);
		}
	}
}

pub struct SkyMaterial {
	pub shader: ProgramObject,
	pub uniform_matrix: i32,
	pub uniform_camera: i32,
	pub uniform_color: i32,
}

impl SkyMaterial {
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, SkyMaterialError> {
		
		let shader_vert = ResourceLocation::from("core/shaders/sky.vert");
		let shader_frag = ResourceLocation::from("core/shaders/sky.frag");
		
		let shader_vert = res.res_as_cstring(&shader_vert)
			.map_err(|err| SkyMaterialError::Resource(err))?;
		
		let shader_frag = res.res_as_cstring(&shader_frag)
			.map_err(|err| SkyMaterialError::Resource(err))?;
		
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(|err| SkyMaterialError::Shader(err))?;
		
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(|err| SkyMaterialError::Shader(err))?;
		
		let shader = ProgramObject::new(gl, "Sky", &smallvec![shader_vert, shader_frag])
			.map_err(|err| SkyMaterialError::Shader(err))?;
		
		let uniform_matrix = shader.get_uniform_location("transform").unwrap();
		let uniform_camera = shader.get_uniform_location("camera").unwrap();
		let uniform_color = shader.get_uniform_location("color").unwrap();
		Ok(Self {
			shader,
			uniform_matrix,
			uniform_camera,
			uniform_color
		})
	}
}

pub enum SkyMaterialError {
	Resource(ResourceError),
	Shader(String),
}

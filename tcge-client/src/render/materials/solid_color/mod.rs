use common::resources::*;
use crate::render::*;

pub const SOLID_COLOR_MATERIAL_FILES: [(&str, &[u8]); 2] = [
	("core/shaders/solid-color.vert", include_bytes!("solid-color.vert")),
	("core/shaders/solid-color.frag", include_bytes!("solid-color.frag")),
];

pub struct SolidColorMaterial {
	pub shader: ProgramObject,
	pub uniform_matrix: UniformLocation,
	pub uniform_color: UniformLocation,
}

impl SolidColorMaterial {
	
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, SolidColorMaterialError> {
		
		let shader_vert = ResourceLocation::from("core/shaders/solid-color.vert");
		let shader_frag = ResourceLocation::from("core/shaders/solid-color.frag");
		
		let shader_vert = res.res_as_cstring(&shader_vert)
			.map_err(SolidColorMaterialError::Resource)?;
		
		let shader_frag = res.res_as_cstring(&shader_frag)
			.map_err(SolidColorMaterialError::Resource)?;
		
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(SolidColorMaterialError::Shader)?;
		
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(SolidColorMaterialError::Shader)?;
		
		let shader = ProgramObject::new(gl, "Sky", &smallvec![shader_vert, shader_frag])
			.map_err(SolidColorMaterialError::Shader)?;
		
		let uniform_matrix = shader.get_uniform_location("transform").unwrap();
		let uniform_color = shader.get_uniform_location("color").unwrap();
		Ok(Self {
			shader,
			uniform_matrix,
			uniform_color,
		})
	}
}

pub enum SolidColorMaterialError {
	Resource(ResourceError),
	Shader(String),
}

use super::resources::ResourceLocation;
use super::resources::ResourceProvider;
use super::resources::ResourceError;
use super::resources::Resources;
use crate::render::*;
use std::rc::Rc;

pub const BLOCKS_MATERIAL_FILES: [(&str, &[u8]); 3] = [
	("core/shaders/blocks.vert", include_bytes!("blocks.vert")),
	("core/shaders/blocks.frag", include_bytes!("blocks.frag")),
	("core/textures/blocks.png", include_bytes!("blocks.png")),
];

pub const BLOCK_SPRITE_FILES: [(&str, &[u8]); 5] = [
	("core/textures/blocks/adm.png", include_bytes!("missingno.png")),
	("core/textures/blocks/adm2.png", include_bytes!("missingno.png")),
	("core/textures/blocks/adm3.png", include_bytes!("missingno.png")),
	("core/textures/blocks/adm4.png", include_bytes!("missingno.png")),
	("core/textures/blocks/adm5.png", include_bytes!("missingno.png")),
];

pub struct BlocksMaterial {
	pub shader: ProgramObject,
	pub atlas: Rc<TextureObject>,
	pub uniform_matrix: UniformLocation,
	pub uniform_atlas: UniformLocation,
	pub uniform_sun: UniformLocation,
}

impl BlocksMaterial {
	pub fn new(gl: &gl::Gl, res: &Resources, atlas: Rc<TextureObject>) -> Result<Self, BlocksMaterialError> {
		debug!("Loading blocks shader...");
		
		let shader_vert = ResourceLocation::from("core/shaders/blocks.vert");
		let shader_frag = ResourceLocation::from("core/shaders/blocks.frag");
		
		let shader_vert = res.res_as_cstring(&shader_vert)
			.map_err(BlocksMaterialError::Resource)?;
		
		let shader_frag = res.res_as_cstring(&shader_frag)
			.map_err(BlocksMaterialError::Resource)?;
		
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(BlocksMaterialError::Shader)?;
		
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(BlocksMaterialError::Shader)?;
		
		let shader = ProgramObject::new(gl, "Blocks", &smallvec![shader_vert, shader_frag])
			.map_err(BlocksMaterialError::Shader)?;
		
		// TODO: Fix error handling
		let uniform_matrix = shader.get_uniform_location("transform").unwrap();
		let uniform_atlas = shader.get_uniform_location("atlas").unwrap();
		let uniform_sun = shader.get_uniform_location("sun").unwrap();
		
		Ok(Self {shader, atlas,
			uniform_matrix,
			uniform_atlas,
			uniform_sun,
		})
	}
}

pub enum BlocksMaterialError {
	Resource(ResourceError),
	Texture(TextureError),
	Shader(String),
}
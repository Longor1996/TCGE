use super::resources::ResourceLocation;
use super::resources::ResourceProvider;
use super::resources::ResourceError;
use super::resources::Resources;
use crate::render::*;

pub const BLOCKS_MATERIAL_FILES: [(&str, &[u8]); 3] = [
	("core/shaders/blocks.vert", include_bytes!("blocks.vert")),
	("core/shaders/blocks.frag", include_bytes!("blocks.frag")),
	("core/textures/blocks.png", include_bytes!("blocks.png")),
];

pub struct BlocksMaterial {
	pub shader: ProgramObject,
	pub atlas: TextureObject,
	pub uniform_matrix: UniformLocation,
	pub uniform_atlas: UniformLocation,
}

impl BlocksMaterial {
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, BlocksMaterialError> {
		debug!("Loading blocks texture...");
		
		let atlas_loc = ResourceLocation::from_str("core/textures/blocks.png");
		let atlas = TextureObjectBuilder::new()
			.wrapping(gl::CLAMP_TO_EDGE)
			.anisotropy(true)
			.filter(gl::NEAREST_MIPMAP_LINEAR, gl::NEAREST)
			.build_from_res(gl, &res, &atlas_loc)
			.map_err(|err| BlocksMaterialError::Texture(err))?;
		
		
		debug!("Loading blocks shader...");
		
		let shader_vert = ResourceLocation::from("core/shaders/blocks.vert");
		let shader_frag = ResourceLocation::from("core/shaders/blocks.frag");
		
		let shader_vert = res.res_as_cstring(&shader_vert)
			.map_err(|err| BlocksMaterialError::Resource(err))?;
		
		let shader_frag = res.res_as_cstring(&shader_frag)
			.map_err(|err| BlocksMaterialError::Resource(err))?;
		
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(|err| BlocksMaterialError::Shader(err))?;
		
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(|err| BlocksMaterialError::Shader(err))?;
		
		let shader = ProgramObject::new(gl, "Blocks", &smallvec![shader_vert, shader_frag])
			.map_err(|err| BlocksMaterialError::Shader(err))?;
		
		// TODO: Fix error handling
		let uniform_matrix = shader.get_uniform_location("transform").unwrap();
		let uniform_atlas = shader.get_uniform_location("atlas").unwrap();
		
		Ok(Self {shader, atlas,
			uniform_matrix,
			uniform_atlas,
		})
	}
}

pub enum BlocksMaterialError {
	Resource(ResourceError),
	Texture(TextureError),
	Shader(String),
}
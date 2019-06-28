use rustc_hash::FxHashMap;
use common::resources::{Resources, ResourceProvider, ResourceLocation};
use super::super::{ShaderObject, ProgramObject, UniformLocation, TextureObject};
use super::TextRendererError;

pub struct TextRendererMaterial {
	pub program: ProgramObject,
	pub pages: FxHashMap<usize, TextureObject>,
	pub uniform_matrix: UniformLocation,
	pub uniform_sdfmap: UniformLocation,
	pub uniform_color:  UniformLocation,
	pub uniform_spread: UniformLocation,
	pub uniform_scale:  UniformLocation,
}

impl TextRendererMaterial {
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, TextRendererError> {
		
		gl.push_debug("Preparing text renderer shaders");
		
		let location = ResourceLocation::from_str("core/shaders/text");
		
		debug!("Loading font shader: {}", location);
		
		trace!("Loading vertex shader...");
		let shader_vert = res.res_as_cstring(&location.add(".vert"))
			.map_err(|e| TextRendererError::Resource(e))?;
		
		trace!("Compiling vertex shader...");
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(|e| TextRendererError::Shader(e))?;
		
		trace!("Loading fragment shader...");
		let shader_frag = res.res_as_cstring(&location.add(".frag"))
			.map_err(|e| TextRendererError::Resource(e))?;
		
		trace!("Compiling fragment shader...");
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(|e| TextRendererError::Shader(e))?;
		
		trace!("Linking program...");
		let program = ProgramObject::new(gl, "Text Renderer", &smallvec![shader_vert, shader_frag])
			.map_err(|e| TextRendererError::Shader(e))?;
		
		trace!("Fetching uniforms...");
		let uniform_matrix = program.get_uniform_location("transform")
			.map_err(|_| TextRendererError::Shader("Missing uniform 'transform'.".to_string()))?;
		
		let uniform_sdfmap = program.get_uniform_location("sdfmap")
			.map_err(|_| TextRendererError::Shader("Missing uniform 'sdfmap'.".to_string()))?;
		
		let uniform_color = program.get_uniform_location("color")
			.map_err(|_| TextRendererError::Shader("Missing uniform 'color'.".to_string()))?;
		
		let uniform_spread = program.get_uniform_location("spread")
			.map_err(|_| TextRendererError::Shader("Missing uniform 'spread'.".to_string()))?;
		
		let uniform_scale = program.get_uniform_location("scale")
			.map_err(|_| TextRendererError::Shader("Missing uniform 'scale'.".to_string()))?;
		
		trace!("Returning material.");
		gl.pop_debug();
		
		Ok(Self {
			program,
			pages: FxHashMap::default(),
			uniform_matrix,
			uniform_sdfmap,
			uniform_color,
			uniform_spread,
			uniform_scale,
		})
	}
}

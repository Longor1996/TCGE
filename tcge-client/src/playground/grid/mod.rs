use common::resources::*;
use cgmath::{Matrix4, Vector3};
use crate::render::*;
use super::*;

pub const GRID_MATERIAL_FILES: [(&str, &[u8]); 2] = [
	("core/shaders/grid.vert", include_bytes!("grid.vert")),
	("core/shaders/grid.frag", include_bytes!("grid.frag")),
];

pub struct GridRenderer {
	gl: gl::Gl,
	size: f32,
	mesh: VertexArray,
	material: GridMaterial,
}

impl GridRenderer {
	pub fn new(gl: &gl::Gl, res: &resources::Resources) -> Result<GridRenderer, GridMaterialError> {
		let shader = GridMaterial::new(gl, res)?;
		let mesh = geometry::geometry_plane_subdivided(gl, 2.0f32.powf(20.0), 64);
		
		// mesh.set_gl_label("PoT Debug Grid");
		
		Ok(GridRenderer {
			gl: gl.clone(),
			size: 256.0,
			mesh,
			material: shader,
		})
	}
	
	pub fn render(&self, camera_transform: &Matrix4<f32>, camera_position: &Vector3<f32>) {
		self.gl.push_debug("Draw Grid");
		
		unsafe {
			self.gl.Enable(gl::BLEND);
			self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			self.gl.Disable(gl::DEPTH_TEST);
		}
		
		let position = camera_position;
		
		let mut grid_transform = Matrix4::from_translation(Vector3::<f32> {
			x: (position.x / self.size).round() * self.size,
			y: 0.0,
			z: (position.z / self.size).round() * self.size,
		});
		
		if position.y < 0.0 {
			// Flip the grid upside-down, by turning it over left.
			grid_transform = grid_transform * Matrix4::from_nonuniform_scale(-1.0, 1.0, 1.0);
		}
		
		let transform = camera_transform * grid_transform;
		
		let shader_grid = &self.material;
		shader_grid.shader.set_used();
		shader_grid.shader.set_uniform_matrix4(shader_grid.uniform_matrix, &transform);
		self.mesh.draw_arrays(&self.gl);
		unsafe {
			self.gl.Enable(gl::DEPTH_TEST);
			self.gl.Disable(gl::BLEND);
		}
		
		self.gl.pop_debug();
	}
}

pub struct GridMaterial {
	pub shader: ProgramObject,
	pub uniform_matrix: UniformLocation,
}

impl GridMaterial {
	pub fn new(gl: &gl::Gl, res: &Resources) -> Result<Self, GridMaterialError> {
		
		let shader_vert = ResourceLocation::from("core/shaders/grid.vert");
		let shader_frag = ResourceLocation::from("core/shaders/grid.frag");
		
		let shader_vert = res.res_as_cstring(&shader_vert)
			.map_err(|err| GridMaterialError::Resource(err))?;
		
		let shader_frag = res.res_as_cstring(&shader_frag)
			.map_err(|err| GridMaterialError::Resource(err))?;
		
		let shader_vert = ShaderObject::new_vertex_shader(gl, &shader_vert)
			.map_err(|err| GridMaterialError::Shader(err))?;
		
		let shader_frag = ShaderObject::new_fragment_shader(gl, &shader_frag)
			.map_err(|err| GridMaterialError::Shader(err))?;
		
		let shader = ProgramObject::new(gl, "Sky", &smallvec![shader_vert, shader_frag])
			.map_err(|err| GridMaterialError::Shader(err))?;
		
		let uniform_matrix = shader.get_uniform_location("transform").unwrap();
		Ok(Self {
			shader,
			uniform_matrix,
		})
	}
}

pub enum GridMaterialError {
	Resource(ResourceError),
	Shader(String),
}

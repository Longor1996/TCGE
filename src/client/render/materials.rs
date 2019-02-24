use super::utility;
use super::super::super::resources::Resources;

pub struct ShaderRandom {
	pub shader_program: utility::Program,
	pub uniform_matrix: i32,
	pub uniform_time: i32,
}
impl ShaderRandom {
	pub fn new(res: &Resources) -> Result<ShaderRandom, utility::Error> {
		let shader_program = utility::Program::from_res(&res, "shaders/triangle")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		let uniform_time = shader_program.uniform_location("time");
		Ok(ShaderRandom {
			shader_program,
			uniform_matrix,
			uniform_time
		})
	}
}

pub struct ShaderSolidColor {
	pub shader_program: utility::Program,
	pub uniform_matrix: i32,
	pub uniform_color: i32,
}
impl ShaderSolidColor {
	pub fn new(res: &Resources) -> Result<ShaderSolidColor, utility::Error> {
		let shader_program = utility::Program::from_res(&res, "shaders/solid-color")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		let uniform_color = shader_program.uniform_location("color");
		Ok(ShaderSolidColor {
			shader_program,
			uniform_matrix,
			uniform_color
		})
	}
}

pub struct ShaderGrid {
	pub shader_program: utility::Program,
	pub uniform_matrix: i32
}
impl ShaderGrid {
	pub fn new(res: &Resources) -> Result<ShaderGrid, utility::Error> {
		let shader_program = utility::Program::from_res(&res, "shaders/grid")?;
		let uniform_matrix = shader_program.uniform_location("transform");
		Ok(ShaderGrid {
			shader_program,
			uniform_matrix
		})
	}
}
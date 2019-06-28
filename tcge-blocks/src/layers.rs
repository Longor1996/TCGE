
#[derive(Copy, Clone, Debug)]
pub struct BlockLayers {
	solid: bool,
	fluid: bool,
	cover: bool,
}

impl Default for BlockLayers {
	fn default() -> Self {
		Self {
			solid: true,
			fluid: false,
			cover: false,
		}
	}
}

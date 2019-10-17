use blocks::Face;
use smallvec::SmallVec;

#[derive(Clone, Debug)]
pub struct BlockModel {
	pub parent: Option<String>,
	// pub inherit_textures: bool,
	// pub inherit_elements: bool,
	pub textures: SmallVec<[String; 1]>,
	pub elements: SmallVec<[BlockModelElement; 1]>,
}

impl Default for BlockModel {
	fn default() -> Self {
		Self {
			parent: None,
			textures: smallvec!("missingno".to_string()),
			elements: smallvec!(BlockModelElement::default())
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct BlockModelElement {
	pub bounds: [f32; 6], // min/max (xyz)
	pub faces: [BlockModelElementFace; 6],
	pub inside: bool,
	pub outside: bool,
}

impl Default for BlockModelElement {
	fn default() -> Self {
		let null_face = BlockModelElementFace {
			uv: [0.0, 0.0, 1.0, 1.0],
			texture: Some(0),
			side: Face::EveryDir,
			cull: true,
		};
		
		Self {
			bounds: [0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
			faces: [
				null_face.with_side(Face::PositiveX),
				null_face.with_side(Face::NegativeX),
				null_face.with_side(Face::PositiveY),
				null_face.with_side(Face::NegativeY),
				null_face.with_side(Face::PositiveZ),
				null_face.with_side(Face::NegativeZ),
			],
			inside: false,
			outside: true,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct BlockModelElementFace {
	pub uv: [f32; 4],
	// pub uv_spin: ...?
	pub texture: Option<u8>,
	
	pub side: Face,
	pub cull: bool,
	// tint: ...?
}

impl BlockModelElementFace {
	pub fn with_side(self, side: Face) -> Self {
		let mut new = self.clone();
		new.side = side;
		new
	}
}

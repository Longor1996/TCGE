use blocks::Face;
use smallvec::SmallVec;

#[derive(Clone, Debug)]
pub struct BlockModel {
	pub parent: Option<String>, // TODO: Let models inherit from each other...
	// pub inherit_textures: bool,
	// pub inherit_elements: bool,
	
	pub textures: SmallVec<[String; 1]>,
	pub elements: SmallVec<[BlockModelElement; 1]>,
}

impl Default for BlockModel {
	/// Creates a model representing a full cube with the `missingno`-texture,
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
	
	// TODO: Implement this here and in the bakery.
	// pub tform: BlockModelTransform,
	
	/// Should geometry for the inside of this element be generated?
	pub inside: bool, // TODO: Make use of this flag.
	
	/// Should geometry for the outside of this element be generated?
	pub outside: bool, // TODO: Make use of this flag.
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
	// pub uv_spin: u8 // TODO: Implement UV rotation...
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

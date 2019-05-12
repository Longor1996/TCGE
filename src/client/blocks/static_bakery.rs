use rustc_hash::FxHashMap;
use crate::resources;
use crate::blocks as blockdef;
use super::super::render;

pub struct StaticBlockBakery {
	baked_blocks: FxHashMap<blockdef::BlockId, Box<BakedBlock>>,
}

impl StaticBlockBakery {
	
	pub fn new(res: &resources::Resources, blockdef: &blockdef::UniverseRef) -> Result<StaticBlockBakery, render::utility::Error> {
		let mut baked_blocks = FxHashMap::default();
		
		for (id, block) in blockdef.list_blocks() {
			let baked_block = Self::bake_block(res, block)?;
			baked_blocks.insert(id.clone(), baked_block);
		}
		
		Ok(StaticBlockBakery {
			baked_blocks
		})
	}
	
	fn bake_block(_res: &resources::Resources, block: &blockdef::Block) -> Result<Box<BakedBlock>, render::utility::Error> {
		let mut sides = FxHashMap::default();
		
		if block.get_id().get_raw_id() == 0 {
			return Ok(Box::new(BasicBakedBlock {
				sides
			}))
		}
		
		const N: f32 = 0.0;
		const S: f32 = 1.0;
		let uv = BlockUv::new_from_pos((block.get_id().get_raw_id()) as u8 - 1, 0);
		
		sides.insert(Face::Ypos, vec![
			(N, S, S, uv.umin, uv.vmin).into(),
			(S, S, S, uv.umax, uv.vmin).into(),
			(S, S, N, uv.umax, uv.vmax).into(),
			(N, S, N, uv.umin, uv.vmax).into(),
		]);
		
		sides.insert(Face::Yneg, vec![
			(N, N, N, uv.umin, uv.vmin).into(),
			(S, N, N, uv.umax, uv.vmin).into(),
			(S, N, S, uv.umax, uv.vmax).into(),
			(N, N, S, uv.umin, uv.vmax).into(),
		]);
		
		sides.insert(Face::Zneg, vec![
			(N, S, N, uv.umin, uv.vmin).into(),
			(S, S, N, uv.umax, uv.vmin).into(),
			(S, N, N, uv.umax, uv.vmax).into(),
			(N, N, N, uv.umin, uv.vmax).into(),
		]);
		
		sides.insert(Face::Zpos, vec![
			(N, N, S, uv.umin, uv.vmin).into(),
			(S, N, S, uv.umax, uv.vmin).into(),
			(S, S, S, uv.umax, uv.vmax).into(),
			(N, S, S, uv.umin, uv.vmax).into(),
		
		]);
		
		sides.insert(Face::Xneg, vec![
			(N, S, S, uv.umin, uv.vmin).into(),
			(N, S, N, uv.umax, uv.vmin).into(),
			(N, N, N, uv.umax, uv.vmax).into(),
			(N, N, S, uv.umin, uv.vmax).into(),
		
		]);
		
		sides.insert(Face::Xpos, vec![
			(S, N, S, uv.umin, uv.vmin).into(),
			(S, N, N, uv.umax, uv.vmin).into(),
			(S, S, N, uv.umax, uv.vmax).into(),
			(S, S, S, uv.umin, uv.vmax).into(),
		]);
		
		Ok(Box::new(BasicBakedBlock {
			sides
		}))
	}
	
	pub fn render_block(&self, context: &BakeryContext, block: &blockdef::BlockState, out: &mut Vec<BakedBlockMeshVertex>) {
		let baked_block = match self.baked_blocks.get(&block.id) {
			Some(bb) => bb,
			None => return
		};
		
		baked_block.build(context, block, out)
	}
	
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct BakeryContext {
	pub occluded: [bool;8]
}

impl BakeryContext {
	pub fn new() -> BakeryContext {
		BakeryContext {
			occluded: [false;8],
		}
	}
	
	pub fn set_occlusion(&mut self, x_pos: bool, y_pos: bool, z_pos: bool, x_neg: bool, y_neg: bool, z_neg: bool, omni: bool) {
		self.occluded[1] = x_pos;
		self.occluded[2] = x_neg;
		self.occluded[3] = y_pos;
		self.occluded[4] = y_neg;
		self.occluded[5] = z_pos;
		self.occluded[6] = z_neg;
		self.occluded[7] = omni;
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

trait BakedBlock {
	fn build(
		&self,
		context: &BakeryContext,
		block: &blockdef::BlockState,
		out: &mut Vec<BakedBlockMeshVertex>
	);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct BasicBakedBlock {
	sides: FxHashMap<Face, Vec<BakedBlockMeshVertex>>,
}

impl BasicBakedBlock {
	fn transfer(&self, context: &BakeryContext, face: Face, out: &mut Vec<BakedBlockMeshVertex>) {
		let face_id = face.id() as usize;
		
		if context.occluded[face_id] {
			return;
		}
		
		let vertices = match self.sides.get(&face) {
			Some(v) => v,
			None => return
		};
		
		for vertex in vertices {
			out.push(*vertex);
		}
	}
}

impl BakedBlock for BasicBakedBlock {
	fn build(
		&self,
		context: &BakeryContext,
		_block: &blockdef::BlockState,
		out: &mut Vec<BakedBlockMeshVertex>
	) {
		self.transfer(context, Face::Xneg, out);
		self.transfer(context, Face::Yneg, out);
		self.transfer(context, Face::Zneg, out);
		self.transfer(context, Face::Xpos, out);
		self.transfer(context, Face::Ypos, out);
		self.transfer(context, Face::Zpos, out);
		self.transfer(context, Face::Omni, out);
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(Debug, Hash, Eq, Copy, Clone)]
pub enum Face {
	Xpos = 1, Xneg = 2,
	Ypos = 3, Yneg = 4,
	Zpos = 5, Zneg = 6,
	Omni = 7
}

impl Face {
	fn id(&self) -> u8 {
		unsafe { ::std::mem::transmute(*self) }
	}
}

impl PartialEq for Face {
	/// Partial equality for the state of a lens, using the `LensState` discriminant.
	fn eq(&self, other: &Face) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct BakedBlockMeshVertex {
	// Geometry
	pub x: f32,
	pub y: f32,
	pub z: f32,
	
	// Texture
	pub u: f32,
	pub v: f32,
}

impl BakedBlockMeshVertex {
	pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Self {
		Self {
			x, y, z, u, v
		}
	}
}

impl From<(f32, f32, f32, f32, f32)> for BakedBlockMeshVertex {
	fn from(other: (f32, f32, f32, f32, f32)) -> Self {
		Self::new(other.0, other.1, other.2, other.3, other.4)
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct BlockUv {
	umin: f32,
	umax: f32,
	vmin: f32,
	vmax: f32,
}

impl BlockUv {
	fn new_from_pos(x: u8, y: u8) -> Self {
		let x = (x as f32) / 16.0;
		let y = (y as f32) / 16.0;
		let s = 1.0 / 16.0;
		Self {
			umin: x,
			umax: x+s,
			vmin: y,
			vmax: y+s,
		}
	}
}

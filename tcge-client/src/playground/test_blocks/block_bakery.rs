use super::*;

pub struct StaticBlockBakery {
	baked_blocks: Vec<Box<BakedBlock>>,
}

impl StaticBlockBakery {
	//
	
	pub fn new(res: &resources::Resources, blocks: &BlocksRef) -> Result<StaticBlockBakery, ()> {
		
		let mut baked_blocks = Vec::with_capacity(blocks.get_blocks().len());
		
		for (id, block) in blocks.get_blocks() {
			let baked_block = Self::bake_block(res, &**block)?;
			baked_blocks.insert(id.raw() as usize, baked_block);
		}
		
		Ok(StaticBlockBakery {
			baked_blocks
		})
	}
	
	fn bake_block(_res: &resources::Resources, block: &Block) -> Result<Box<BakedBlock>, ()> {
		
		if block.get_name() == "air" {
			return Ok(Box::new(EmptyBakedBlock{}));
		}
		
		let mut sides: [smallvec::SmallVec<[BakedBlockMeshFace;6]>; 8] = [
			smallvec![],
			smallvec![],
			smallvec![],
			smallvec![],
			smallvec![],
			smallvec![],
			smallvec![],
			smallvec![],
		];
		
		if block.get_id().raw() == 0 {
			return Ok(Box::new(BasicBakedBlock {
				sides
			}))
		}
		
		const N: f32 = 0.0;
		const S: f32 = 1.0;
		let uv = BlockUv::new_from_pos((block.get_id().raw()) as u8 - 1, 0);
		
		let n = (0.0, 1.0, 0.0);
		sides[Face::Ypos.uid()].push((
			(N, S, S, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
			(S, S, S, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
			(S, S, N, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
			(N, S, N, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
		).into());
		
		let n = (0.0, -1.0, 0.0);
		sides[Face::Yneg.uid()].push((
			(N, N, N, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
			(S, N, N, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
			(S, N, S, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
			(N, N, S, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
		).into());
		
		let n = (0.0, 0.0, -1.0);
		sides[Face::Zneg.uid()].push((
			(N, S, N, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
			(S, S, N, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
			(S, N, N, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
			(N, N, N, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
		).into());
		
		let n = (0.0, 0.0, 1.0);
		sides[Face::Zpos.uid()].push((
			(N, N, S, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
			(S, N, S, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
			(S, S, S, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
			(N, S, S, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
		).into());
		
		let n = (-1.0, 0.0, 0.0);
		sides[Face::Xneg.uid()].push((
			(N, S, S, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
			(N, S, N, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
			(N, N, N, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
			(N, N, S, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
		).into());
		
		let n = (1.0, 0.0, 0.0);
		sides[Face::Xpos.uid()].push((
			(S, N, S, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
			(S, N, N, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
			(S, S, N, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
			(S, S, S, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
		).into());
		
		Ok(Box::new(BasicBakedBlock {
			sides
		}))
	}
	
	pub fn render_block(&self, context: &BakeryContext, block: &BlockState, out: &mut FnMut(&BakedBlockMeshFace)) {
		match self.baked_blocks.get(block.id.raw() as usize) {
			Some(bb) => bb.build(context, block, out),
			None => return
		};
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
		block: &BlockState,
		out: &mut FnMut(&BakedBlockMeshFace)
	);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct BasicBakedBlock {
	sides: [smallvec::SmallVec<[BakedBlockMeshFace;6]>;8],
}

impl BasicBakedBlock {
	fn transfer(&self, context: &BakeryContext, face: Face, out: &mut FnMut(&BakedBlockMeshFace)) {
		let face_id = face.id() as usize;
		
		if context.occluded[face_id] {
			return;
		}
		
		for vertex in self.sides[face_id].iter() {
			out(&vertex);
		}
	}
}

impl BakedBlock for BasicBakedBlock {
	fn build(
		&self,
		context: &BakeryContext,
		_block: &BlockState,
		out: &mut FnMut(&BakedBlockMeshFace)
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

struct EmptyBakedBlock;

impl BakedBlock for EmptyBakedBlock {
	fn build(&self, _context: &BakeryContext, _block: &BlockState, _out: &mut FnMut(&BakedBlockMeshFace)) {
		// Don't do anything what-so-ever.
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
	
	fn uid(&self) -> usize {
		self.id() as usize
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
pub struct BakedBlockMeshFace {
	pub a: BakedBlockMeshVertex,
	pub b: BakedBlockMeshVertex,
	pub c: BakedBlockMeshVertex,
	pub d: BakedBlockMeshVertex
}

impl From<(BakedBlockMeshVertex, BakedBlockMeshVertex, BakedBlockMeshVertex, BakedBlockMeshVertex)> for BakedBlockMeshFace {
	fn from(vertices: (BakedBlockMeshVertex, BakedBlockMeshVertex, BakedBlockMeshVertex, BakedBlockMeshVertex)) -> Self {
		Self {
			a: vertices.0,
			b: vertices.1,
			c: vertices.2,
			d: vertices.3,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub struct BakedBlockMeshVertex {
	// Geometry
	pub x: f32,
	pub y: f32,
	pub z: f32,
	
	// Texture
	pub u: f32,
	pub v: f32,
	
	// Normal
	pub nx: f32,
	pub ny: f32,
	pub nz: f32,
}

impl BakedBlockMeshVertex {
	pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32, nx: f32, ny: f32, nz: f32) -> Self {
		Self {
			x, y, z, u, v, nx, ny, nz
		}
	}
}

impl From<(f32, f32, f32, f32, f32, f32, f32, f32)> for BakedBlockMeshVertex {
	fn from(other: (f32, f32, f32, f32, f32, f32, f32, f32)) -> Self {
		Self::new(other.0, other.1, other.2, other.3, other.4, other.5, other.6, other.7)
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

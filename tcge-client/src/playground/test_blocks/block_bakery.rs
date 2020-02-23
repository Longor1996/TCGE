use super::*;
use blocks::Face;
use std::borrow::Borrow;
use std::rc::Rc;

pub type StaticBlockBakeryRef = Rc<StaticBlockBakery>;

pub struct StaticBlockBakery {
	baked_blocks: Vec<Box<dyn BakedBlock>>,
}

impl StaticBlockBakery {
	//
	
	pub fn new(_res: &resources::Resources, blocks: &BlocksRef) -> Result<StaticBlockBakery, ()> {
		
		// TODO: The texture atlas should be built outside the bakery...
		let mut textures: FxHashMap<String, BlockUv> = FxHashMap::default();
		textures.insert("missingno".to_string(), BlockUv::unit());
		textures.insert("tex1".to_string(), BlockUv::new_from_pos(0, 0));
		textures.insert("tex2".to_string(), BlockUv::new_from_pos(1, 0));
		textures.insert("tex3".to_string(), BlockUv::new_from_pos(2, 0));
		textures.insert("tex4".to_string(), BlockUv::new_from_pos(3, 0));
		textures.insert("tex5".to_string(), BlockUv::new_from_pos(4, 0));
		
		// --- Create rendering-table for all blocks...
		let mut baked_blocks: Vec<Box<dyn BakedBlock>> = Vec::with_capacity(blocks.get_blocks().len() + 1);
		for _ in 0..blocks.get_blocks().len() {
			// ...and fill it with EmptyBakedBlock's.
			baked_blocks.push(Box::new(EmptyBakedBlock {}));
		}
		
		// --- Go trough all blocks and bake them.
		for (id, block) in blocks.get_blocks() {
			
			if block.get_name() == "air" {
				// Do not bake air.
				continue;
			}
			
			// TODO: Load model from a file, BEFORE the bakery, somehow...
			let mut block_model = BlockModel::default();
			block_model.textures[0] = format!("tex{}", id.raw());
			
			// Bake the model for the block...
			let baked_block = Self::bake_model(block.borrow(), &block_model, &textures);
			
			// ...and place it into the bakery's list.
			baked_blocks[id.raw() as usize] = baked_block;
		}
		
		// Nothing went wrong, yay!
		Ok(StaticBlockBakery {
			baked_blocks
		})
	}
	
	fn bake_model(block: &dyn Block, block_model: &BlockModel, textures: &FxHashMap<String, BlockUv>) -> Box<dyn BakedBlock> {
		
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
		
		for element in &block_model.elements {
			
			let [min_x, min_y, min_z, max_x, max_y, max_z] = element.bounds.clone();
			
			// TODO: What about rotation?
			
			{ // Positive Y: Top
				let i_face = Face::PositiveY;
				let face = element.faces[i_face.uid() - 1];
				let n = i_face.normal();
				
				if let Some(texture_id) = face.texture {
					let texture = &block_model.textures[texture_id as usize];
					let uv = textures.get(texture).expect("valid texture reference");
					let uv = uv.subset(&face.uv);
					
					sides[if face.cull { i_face.uid()} else {Face::EveryDir.uid()}].push((
						(min_x, max_y, max_z, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, max_y, max_z, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, max_y, min_z, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
						(min_x, max_y, min_z, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
					).into());
				}
			}
			
			{ // Negative Y: Bottom
				let i_face = Face::NegativeY;
				let face = element.faces[i_face.uid() - 1];
				let n = i_face.normal();
				
				if let Some(texture_id) = face.texture {
					let texture = &block_model.textures[texture_id as usize];
					let uv = textures.get(texture).expect("valid texture reference");
					
					sides[if face.cull { i_face.uid()} else {Face::EveryDir.uid()}].push((
						(min_x, min_y, min_z, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, min_y, min_z, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, min_y, max_z, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
						(min_x, min_y, max_z, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
					).into());
				}
			}
			
			{ // Negative X
				let i_face = Face::NegativeX;
				let face = element.faces[i_face.uid() - 1];
				let n = i_face.normal();
				
				if let Some(texture_id) = face.texture {
					let texture = &block_model.textures[texture_id as usize];
					let uv = textures.get(texture).expect("valid texture reference");
					
					sides[if face.cull { i_face.uid()} else {Face::EveryDir.uid()}].push((
						(min_x, max_y, max_z, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
						(min_x, max_y, min_z, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
						(min_x, min_y, min_z, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
						(min_x, min_y, max_z, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
					).into());
				}
			}
			
			{ // Positive X
				let i_face = Face::PositiveX;
				let face = element.faces[i_face.uid() - 1];
				let n = i_face.normal();
				
				if let Some(texture_id) = face.texture {
					let texture = &block_model.textures[texture_id as usize];
					let uv = textures.get(texture).expect("valid texture reference");
					
					sides[if face.cull { i_face.uid()} else {Face::EveryDir.uid()}].push((
						(max_x, min_y, max_z, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, min_y, min_z, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, max_y, min_z, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
						(max_x, max_y, max_z, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
					).into());
				}
			}
			
			{ // Negative Z
				let i_face = Face::NegativeZ;
				let face = element.faces[i_face.uid() - 1];
				let n = i_face.normal();
				
				if let Some(texture_id) = face.texture {
					let texture = &block_model.textures[texture_id as usize];
					let uv = textures.get(texture).expect("valid texture reference");
					
					sides[if face.cull { i_face.uid()} else {Face::EveryDir.uid()}].push((
						(min_x, max_y, min_z, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, max_y, min_z, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, min_y, min_z, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
						(min_x, min_y, min_z, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
					).into());
				}
			}
			
			{ // Positive Z
				let i_face = Face::PositiveZ;
				let face = element.faces[i_face.uid() - 1];
				let n = i_face.normal();
				
				if let Some(texture_id) = face.texture {
					let texture = &block_model.textures[texture_id as usize];
					let uv = textures.get(texture).expect("valid texture reference");
					
					sides[if face.cull { i_face.uid()} else {Face::EveryDir.uid()}].push((
						(min_x, min_y, max_z, uv.umin, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, min_y, max_z, uv.umax, uv.vmin, n.0, n.1, n.2).into(),
						(max_x, max_y, max_z, uv.umax, uv.vmax, n.0, n.1, n.2).into(),
						(min_x, max_y, max_z, uv.umin, uv.vmax, n.0, n.1, n.2).into(),
					).into());
				}
			}
			
		}
		
		Box::new(BasicBakedBlock {
			sides
		})
	}
	
	pub fn render_block(&self, context: &BakeryContext, block: &BlockState, out: &mut dyn FnMut(&BakedBlockMeshFace)) {
		match self.baked_blocks.get(block.id.raw() as usize) {
			Some(bb) => bb.build(context, block, out),
			None => {}
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
		out: &mut dyn FnMut(&BakedBlockMeshFace)
	);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct BasicBakedBlock {
	sides: [smallvec::SmallVec<[BakedBlockMeshFace;6]>;8],
}

impl BasicBakedBlock {
	fn transfer(&self, context: &BakeryContext, face: Face, out: &mut dyn FnMut(&BakedBlockMeshFace)) {
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
		out: &mut dyn FnMut(&BakedBlockMeshFace)
	) {
		self.transfer(context, Face::NegativeX, out);
		self.transfer(context, Face::NegativeY, out);
		self.transfer(context, Face::NegativeZ, out);
		self.transfer(context, Face::PositiveX, out);
		self.transfer(context, Face::PositiveY, out);
		self.transfer(context, Face::PositiveZ, out);
		self.transfer(context, Face::EveryDir, out);
	}
}


////////////////////////////////////////////////////////////////////////////////////////////////////

struct EmptyBakedBlock;

impl BakedBlock for EmptyBakedBlock {
	fn build(&self, _context: &BakeryContext, _block: &BlockState, _out: &mut dyn FnMut(&BakedBlockMeshFace)) {
		// Don't do anything what-so-ever.
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

// TODO: Move this out of the bakery...
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
	
	fn unit() -> Self {
		Self {
			umin: 0.0, umax: 1.0,
			vmin: 0.0, vmax: 1.0,
		}
	}
	
	fn subset(&self, uv: &[f32; 4]) -> Self {
		Self {
			umin: Self::lerp(self.umin, self.umax, uv[0]),
			vmin: Self::lerp(self.vmin, self.vmax, uv[1]),
			umax: Self::lerp(self.umin, self.umax, uv[2]),
			vmax: Self::lerp(self.vmin, self.vmax, uv[3]),
		}
	}
	
	fn lerp(v0: f32, v1: f32, x: f32) -> f32 {
		(1.0 - x) * v0 + x * v1
	}
}

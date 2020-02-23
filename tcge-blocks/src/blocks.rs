use rustc_hash::FxHashMap;
use super::BlockId;
use super::Block;
use super::BlockState;
use super::BlockLayers;
use std::rc::Rc;

pub type BlocksRef = std::rc::Rc<Blocks>;

pub struct Blocks {
	blocks: FxHashMap<BlockId, Box<dyn Block>>,
	names: FxHashMap<String, BlockId>,
	defaults: FxHashMap<BlockId, BlockState>,
}

impl Blocks {
	pub fn new() -> Self {
		let mut new = Self {
			blocks: FxHashMap::default(),
			names: FxHashMap::default(),
			defaults: FxHashMap::default(),
		};
		
		use super::block::simple::SimpleBlock;
		new.register_block(Box::new(SimpleBlock::new(BlockId::new(0), "air", BlockLayers::default())));
		new.register_block(Box::new(SimpleBlock::new(BlockId::new(1), "adm", BlockLayers::default())));
		
		new.register_block(Box::new(SimpleBlock::new(BlockId::new(2), "adm2", BlockLayers::default())));
		new.register_block(Box::new(SimpleBlock::new(BlockId::new(3), "adm3", BlockLayers::default())));
		new.register_block(Box::new(SimpleBlock::new(BlockId::new(4), "adm4", BlockLayers::default())));
		new.register_block(Box::new(SimpleBlock::new(BlockId::new(5), "adm5", BlockLayers::default())));
		
		new
	}
	
	pub fn register_block(&mut self, block: Box<dyn Block>) {
		let id = block.get_id();
		if let Some(_) = self.names.insert(block.get_name().to_string(), id) {
			panic!("Cannot register block '{}': Name is already taken.", block.get_name());
		}
		
		self.defaults.insert(id, block.get_default_state());
		self.blocks.insert(id, block);
	}
	
	pub fn to_ref(self) -> Rc<Self> {
		Rc::new(self)
	}
}

impl Blocks {
	pub fn get_blocks(&self) -> &FxHashMap<BlockId, Box<dyn Block>> {
		&self.blocks
	}
	
	pub fn get_block_by_id(&self, id: BlockId) -> Option<&Box<dyn Block>> {
		self.blocks.get(&id)
	}
	
	pub fn get_block_by_id_unchecked(&self, id: BlockId) -> &Box<dyn Block> {
		self.get_block_by_id(id).expect("Could not find block-type.")
	}
	
	pub fn get_block_by_name(&self, name: &str) -> Option<&Box<dyn Block>> {
		for (_id, block) in self.blocks.iter() {
			if block.get_name() == name {
				return Some(block)
			}
		}
		
		None
	}
	
	pub fn get_block_by_name_unchecked(&self, name: &str) -> &Box<dyn Block> {
		self.get_block_by_name(name).expect("Could not find block-type.")
	}
}
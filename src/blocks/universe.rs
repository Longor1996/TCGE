//! This file defines the 'universe' of blocks.
//! Its simply the definition of blocks and their states.
//!
//! TODO: Load the universe from a file.

use rustc_hash::FxHashMap;

pub struct Universe {
	blocks: FxHashMap<BlockId, Block>,
}

impl Universe {
	pub fn get_block_by_name(&self, name: &str) -> Option<&Block> {
		for (_id, block) in self.blocks.iter() {
			if block.name == name {
				return Some(block)
			}
		}
		
		None
	}
	
	pub fn get_block_by_id(&self, id: BlockId) -> &Block {
		return &self.blocks[&id];
	}
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Eq,Hash,Clone)]
pub struct BlockId {
	id: u16
}

impl PartialEq for BlockId {
	fn eq(&self, other: &BlockId) -> bool {
		self.id == other.id
	}
}

////////////////////////////////////////////////////////////////////////////////

pub struct Block {
	id: BlockId,
	name: String,
	default_state: BlockState,
}

impl Block {
	pub fn get_default_state(&self) -> BlockState {
		return self.default_state.clone();
	}
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct BlockState {
	data: ()
}

////////////////////////////////////////////////////////////////////////////////

pub fn define_universe() -> Universe {
	let mut blocks = FxHashMap::default();
	
	let air = Block {
		id: BlockId {id: 0},
		name: "air".to_string(),
		default_state: BlockState {
			data: ()
		}
	};
	blocks.insert(air.id.clone(), air);
	
	let bedrock = Block {
		id: BlockId {id: 1},
		name: "bedrock".to_string(),
		default_state: BlockState {
			data: ()
		}
	};
	blocks.insert(bedrock.id.clone(), bedrock);
	
	return Universe {
		blocks
	}
}

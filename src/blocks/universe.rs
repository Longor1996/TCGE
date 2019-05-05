//! This file defines the 'universe' of blocks.
//! Its simply the definition of blocks and their states.
//!
//! TODO: Load the universe from a file.

use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Universe {
	blocks: FxHashMap<BlockId, Block>,
}

impl Universe {
	
	fn new() -> Universe {
		Universe {
			blocks: FxHashMap::default()
		}
	}
	
	pub fn get_block_by_name(&self, name: &str) -> Option<&Block> {
		for (_id, block) in self.blocks.iter() {
			if block.name == name {
				return Some(block)
			}
		}
		
		None
	}
	
	pub fn get_block_by_name_unchecked(&self, name: &str) -> &Block {
		self.get_block_by_name(name).unwrap()
	}
	
	pub fn get_block_by_id(&self, id: BlockId) -> &Block {
		return &self.blocks[&id];
	}
}

pub type UniverseRef = Rc<Universe>;

////////////////////////////////////////////////////////////////////////////////

/// Immutable handle to a specific type of block in a universe.
#[derive(Eq, Hash, Copy, Clone)]
pub struct BlockId {
	id: u16 // Not public; must stay immutable.
}

impl BlockId {
	pub fn new(id: u16) -> BlockId {
		BlockId {id}
	}
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

#[derive(Copy, Clone, Eq)]
pub struct BlockState {
	pub id: BlockId,
	pub data: ()
}

impl PartialEq for BlockState {
	fn eq(&self, other: &BlockState) -> bool {
		self.id == other.id
	}
}

////////////////////////////////////////////////////////////////////////////////

pub fn define_universe(
	_config: &toml::value::Table
) -> UniverseRef {
	let mut universe = Universe::new();
	
	let air_id = BlockId::new(0);
	let air = Block {
		id: air_id,
		name: "air".to_string(),
		default_state: BlockState {
			id: air_id,
			data: ()
		}
	};
	universe.blocks.insert(air.id, air);
	
	let bedrock_id = BlockId::new(1);
	let bedrock = Block {
		id: bedrock_id,
		name: "bedrock".to_string(),
		default_state: BlockState {
			id: bedrock_id,
			data: ()
		}
	};
	universe.blocks.insert(bedrock.id, bedrock);
	
	return Rc::new(universe)
}

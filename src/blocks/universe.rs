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
	
	fn register_block(&mut self, name: &str) {
		let next_id = self.blocks.len();
		let block_id = BlockId::new(next_id);
		
		let default_state = BlockState {
			id: block_id, data: ()
		};
		
		let block = Block {
			id: block_id,
			name: name.to_string(),
			default_state
		};
		
		self.blocks.insert(block.id, block);
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
#[derive(Eq, Hash, Copy, Clone, Debug)]
pub struct BlockId {
	id: u16 // Not public; must stay immutable.
}

impl BlockId {
	pub fn new(id: usize) -> BlockId {
		BlockId {id: id as u16}
	}
	
	pub fn get_raw_id(&self) -> usize {
		self.id as usize
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
	pub fn get_name(&self) -> &str {
		return self.name.as_str();
	}
	
	pub fn get_default_state(&self) -> BlockState {
		return self.default_state.clone();
	}
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, Debug)]
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
	
	universe.register_block("air");
	universe.register_block("bedrock");
	universe.register_block("bedrock2");
	universe.register_block("bedrock3");
	
	return Rc::new(universe)
}

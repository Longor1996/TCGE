/*
	This file defines the 'universe' of blocks.
	Its simply the definition of blocks and their states.
	
	TODO: Load the universe from a file.
*/

pub struct BlockUniverse {
	blocks: Vec<Block>,
}

impl BlockUniverse {
	pub fn get_block_by_id(&self, id: usize) -> &Block {
		return &self.blocks[id];
	}
}

pub struct Block {
	name: String,
	default_state: BlockState,
}

impl Block {
	pub fn get_default_state(&self) -> BlockState {
		return self.default_state.clone();
	}
}

#[derive(Clone)]
pub struct BlockState {
	data: u32
}

////////////////////////////////////////////////////////////////////////////////

pub fn define_universe() -> BlockUniverse {
	let air = Block {
		name: "air".to_string(),
		default_state: BlockState {
			data: 0
		}
	};
	
	let bedrock = Block {
		name: "bedrock".to_string(),
		default_state: BlockState {
			data: 0
		}
	};
	
	return BlockUniverse {
		blocks: vec![
			air,
			bedrock
		]
	}
}

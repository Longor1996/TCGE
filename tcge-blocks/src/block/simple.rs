//! The simplest way to define a new type of block.
//! This kind of block has exactly one state.

use super::BlockId;
use super::BlockLayers;
use super::BlockState;

pub struct SimpleBlock {
	id: BlockId,
	name: String,
	layers: BlockLayers,
	default: BlockState,
}

impl SimpleBlock {
	pub fn new(id: BlockId, name: &str, layers: BlockLayers) -> Self {
		let name = name.to_string();
		let default = BlockState {
			id, data: ()
		};
		
		Self {
			id,
			name,
			layers,
			default,
		}
	}
}

impl super::Block for SimpleBlock {
	fn get_id(&self) -> BlockId {
		self.id
	}
	
	fn get_name(&self) -> &str {
		self.name.as_str()
	}
	
	fn get_layers(&self) -> BlockLayers {
		self.layers
	}
	
	fn get_default_state(&self) -> BlockState {
		self.default
	}
}

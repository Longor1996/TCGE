//! Definition of types of block.

use super::BlockId;
use super::BlockLayers;

pub trait Block {
	fn get_id(&self) -> BlockId;
	fn get_name(&self) -> &str;
	fn get_layers(&self) -> BlockLayers;
	fn get_default_state(&self) -> BlockState;
}

impl PartialEq for dyn Block {
	fn eq(&self, other: &dyn Block) -> bool {
		self.get_id() == other.get_id()
	}
}

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

// Concrete implementations
pub mod simple;

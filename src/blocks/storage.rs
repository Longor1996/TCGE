//! Various types of storage for block-data.

use super::universe::BlockState;

pub trait BlockStorage {
	fn get(&self, location: usize) -> BlockState;
	fn set(&mut self, location: usize, state: &BlockState);
}

////////////////////////////////////////////////////////////////////////////////

pub struct SimpleBlockStorage {
	blocks: Vec<BlockState>,
}

impl BlockStorage for SimpleBlockStorage {
	fn get(&self, location: usize) -> BlockState {
		return self.blocks[location].clone();
	}
	fn set(&mut self, location: usize, state: &BlockState) {
		self.blocks[location] = state.clone();
	}
}

type BlockIdRaw = u16;

/// Immutable handle to a specific type of block.
#[derive(Eq, Hash, Copy, Clone, Debug)]
pub struct BlockId {
	inner: BlockIdRaw // Not public; must stay immutable.
}

impl BlockId {
	pub fn new(id: usize) -> BlockId {
		BlockId { inner: id as BlockIdRaw}
	}
	
	pub fn raw(&self) -> BlockIdRaw {
		self.inner as BlockIdRaw
	}
}

impl PartialEq for BlockId {
	fn eq(&self, other: &BlockId) -> bool {
		self.inner == other.inner
	}
}

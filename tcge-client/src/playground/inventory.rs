pub struct Inventory {
	pub(crate) block: Option<blocks::BlockState>,
}

impl Default for Inventory {
	fn default() -> Self {
		Self {
			block: None,
		}
	}
}
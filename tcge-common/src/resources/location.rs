pub struct ResourceLocation {
	pub inner: String
}

impl ResourceLocation {
	pub fn from_str(path: &str) -> Self {
		Self::from_string(path.to_string())
	}
	
	pub fn from_string(path: String) -> Self {
		Self {
			inner: path
		}
	}
	
	pub fn pre(&self, lhs: &str) -> Self {
		Self {
			inner: lhs.to_string() + &self.inner
		}
	}
	
	pub fn add(&self, rhs: &str) -> Self {
		Self {
			inner: self.inner.clone() + rhs
		}
	}
}

impl From<&str> for ResourceLocation {
	fn from(path: &str) -> Self {
		Self::from_str(path)
	}
}

impl From<String> for ResourceLocation {
	fn from(path: String) -> Self {
		Self::from_string(path)
	}
}

impl std::fmt::Display for ResourceLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.inner)
	}
}

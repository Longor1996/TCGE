use super::Handler;

/// Collection type for `Node` instances.
pub type Nodes = super::FxHashMap<NodeId, Node>;

// Implementation details regarding nodes.
impl super::Backbone {
	
	pub fn root_get_id(&self) -> NodeId {
		self.root_id
	}
	
	pub fn root_get(&self) -> &Node {
		self.nodes.get(&self.root_id).unwrap()
	}
	
	pub fn node_new(&mut self, parent: NodeId, name: &str, handler: Option<Box<dyn Handler>>) -> Result<NodeId, ()> {
		// Ensure parent ID is valid
		if ! self.nodes.contains_key(&parent) {
			return Err(())
		}
		
		// Get new ID for node
		self.counter += 1;
		let node_id = NodeId::new(self.counter);
		
		// Actually create node
		let node = Node::new(node_id, name, parent);
		
		// Add node to tree
		self.nodes.insert(node_id, node);
		
		// Has handler? Add handler.
		if let Some(handler) = handler {
			self.handlers.insert(node_id, handler);
		}
		
		return Ok(node_id)
	}
	
	pub fn path_to_string(&self, path: &[NodeId]) -> Result<String, ()> {
		let mut path_str = String::new();
		
		for item in &path[1..] {
			let node = self.nodes.get(item);
			
			if let Some(node) = node {
				path_str += "/";
				path_str += node.name.as_str();
			} else {
				return Err(());
			}
		}
		
		if path_str.is_empty() && !path.is_empty() {
			path_str += "/";
		}
		
		Ok(path_str)
	}
}

////////////////////////////////////////////////////////////////////////////////

type InternalNodeId = usize;

/// A reference to a node in the backbone, that may not exist.
#[derive(Eq, Hash, Clone, Copy, Debug)]
pub struct NodeId {
	inner: InternalNodeId
}

impl NodeId {
	pub fn new(id: InternalNodeId) -> Self {
		Self {
			inner: id
		}
	}
	
	pub fn get_inner(&self) -> InternalNodeId {
		self.inner
	}
}

impl PartialEq for NodeId {
	fn eq(&self, other: &Self) -> bool {
		self.inner == other.inner
	}
}

impl std::fmt::Display for NodeId {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		write!(f, "#{}", self.inner)
	}
}

////////////////////////////////////////////////////////////////////////////////

/// A node of the backbone.
///
/// Consists of a own `NodeId`, name and a parent `NodeId`.
pub struct Node {
	id: NodeId,
	name: String,
	parent: NodeId,
}

impl Node {
	pub fn new(id: NodeId, name: &str, parent: NodeId) -> Self {
		Self {
			id,
			name: name.to_string(),
			parent,
		}
	}
	
	pub fn get_id(&self) -> NodeId {
		self.id
	}
	
	pub fn get_parent_id(&self) -> NodeId {
		self.parent
	}
	
	pub fn get_name(&self) -> &str {
		self.name.as_str()
	}
}
















// TODO: Re-Implement the node "handlers".
















use super::{NodeId, Nodes, Node, Event};

/// This function is called when the backbone attempts to change its path.
pub fn update_path(
	nodes: &Nodes,
	dst_path: &str,
	dst_offset: &mut usize,
	src_path: &[NodeId]
) -> PathChange {
	// If we are not within the backbone, move to the root node (#0).
	if src_path.is_empty() {
		return PathChange::ToRoot
	}
	
	// Parsing of starting location only happens when `offset = 0`
	if *dst_offset == 0 {
		if dst_path.starts_with("/") {
			// Bubble until you hit the root
			if src_path.len() > 1 {
				return PathChange::ToSuper;
			}
			
			*dst_offset += 1;
		}
		
		if dst_path.starts_with("./") {
			*dst_offset += 2;
			return PathChange::ToSelf
		}
		
		if dst_path.starts_with("../") {
			*dst_offset += 3;
			return PathChange::ToSuper
		}
	}
	
	// Slice away everything before the offset
	let mut path = dst_path.split_at(*dst_offset).1;
	
	// Have we already reached the end?
	if path.len() == 0 {
		return PathChange::End;
	}
	
	// Slice away unnecessary slashes
	while path.starts_with("/") {
		*dst_offset += 1;
		path = &path[1..];
	}
	
	if path.starts_with("./") {
		*dst_offset += 2;
		return PathChange::ToSelf
	}
	
	if path.starts_with("../") {
		*dst_offset += 3;
		return PathChange::ToSuper
	}
	
	let current = src_path.last().expect("src_path should not be empty due to the first guard-clause");
	let current = nodes.get(current);
	
	let current = match current {
		Some(x) => x,
		None => return PathChange::Error(format!("Could not resolve current.")),
	};
	
	let end = path.find("/")
		.unwrap_or(path.len());
	
	let name = &path[..end];
	
	let mut next: Option<&Node> = None;
	for (_, node) in nodes.iter() {
		if node.get_name() != name {
			continue;
		}
		
		if node.get_parent_id() != current.get_id() {
			continue;
		}
		
		next = Some(node);
	}
	
	let next = match next {
		None => return PathChange::Error(format!("Could not find node: {}", name)),
		Some(x) => x,
	};
	
	*dst_offset += end;
	return PathChange::ToNode(next.get_id());
}

////////////////////////////////////////////////////////////////////////////////

/// An intended change to the backbones current location.
pub enum PathChange {
	/// `/`
	ToRoot,
	
	/// `./`
	ToSelf,
	
	/// `../`
	ToSuper,
	
	/// `NAME`
	ToNode(NodeId),
	
	/// Completion: Failure
	Error(String),
	
	/// Completion: Success
	End
}

impl std::fmt::Display for PathChange {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			PathChange::ToRoot => write!(fmt, "/"),
			PathChange::ToSelf => write!(fmt, "./"),
			PathChange::ToSuper => write!(fmt, "../"),
			PathChange::ToNode(x) => write!(fmt, "/{}", x),
			PathChange::Error(x) => write!(fmt, "!Err({})", x),
			PathChange::End => write!(fmt, "!Ok"),
		}
	}
}


////////////////////////////////////////////////////////////////////////////////

/// The state of the backbones location.
pub enum State {
	Idle,
	Move(String, usize),
	Stop(Option<String>),
	Fire(Box<Event>),
}

impl State {
	pub fn can_replace(&self) -> bool {
		match self {
			State::Idle => true,
			State::Move(_, _) => false,
			State::Stop(_) => false,
			State::Fire(_) => false,
		}
	}
}

impl PartialEq for State {
	fn eq(&self, other: &State) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}

impl std::fmt::Display for State {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			State::Idle => write!(fmt, "Idle"),
			State::Move(path, offset) => write!(fmt, "Move({}, {})", path, offset),
			State::Fire(event) => write!(fmt, "Fire({})", event.get_type_name()),
			State::Stop(_reason) => write!(fmt, "Stop(?)"),
		}
	}
}

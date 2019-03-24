use core::borrow::{BorrowMut};

extern crate rustc_hash;

pub mod node;
pub mod lens;
pub mod event;

pub struct Router {
	pub lenses: lens::RouterLenses,
	pub nodes: node::RouterNodes,
}

/// Functions for building the router.
impl Router {
	
	pub fn new() -> Router {
		Router {
			lenses: lens::RouterLenses::new(),
			nodes: node::RouterNodes::new(),
		}
	}
	
	pub fn new_lens(&mut self, name: &str, constructor: &Fn(&mut lens::Lens) -> Option<Box<lens::LensHandler>>) {
		let mut lens = lens::Lens {
			name: name.to_string(),
			path_str: "".to_string(),
			path: vec![],
			state: lens::LensState::Idle,
		};
		
		let handler = constructor(&mut lens).unwrap_or(Box::new(lens::NULL_HANDLER));
		
		self.lenses.lenses.push(lens);
		self.lenses.handlers.push(handler);
	}
	
	pub fn new_node(&mut self, name: &str, parent: Option<usize>, constructor: &Fn(&mut node::Node)) -> usize {
		let parent = parent.or(Some(0));
		let id = self.nodes.next_id();
		
		let mut node = node::Node {
			id,
			parent,
			name: name.to_string(),
		};
		
		constructor(&mut node);
		
		self.nodes.nodes.insert(node.id, node);
		return id;
	}
}

// Router update handling
impl Router {
	pub fn update(&mut self) -> bool {
		let mut events: Vec<(usize, Box<event::Event>)> = vec![];
		
		for (pos, lens) in self.lenses.lenses.iter_mut().enumerate() {
			
			if lens.path.is_empty() {
				// All lenses must be at least at root-level
				lens.state = lens::LensState::Moving("/".to_string(), 0);
			}
			
			// Ignore all idle and destroying lenses
			if lens.state == lens::LensState::Idle || lens.state == lens::LensState::Destruction {
				continue
			}
			
			let new_state = match lens.state.borrow_mut() {
				lens::LensState::Moving(path, offset) => {
					
					let step = Router::path_next(
						&self.nodes,
						path,
						offset,
						&lens.path
					);
					
					let new_state = match step {
						PathItem::ToSelf => None,
						PathItem::ToRoot => {
							lens.path.clear();
							lens.path.push(0);
							None
						},
						PathItem::ToSuper => {
							lens.path.pop();
							None
						},
						PathItem::ToNode(x) => {
							lens.path.push(x);
							None
						},
						PathItem::Error(_e) => {
							let event = lens::LensMoveEvent::Aborted;
							events.push((pos, Box::new(event)));
							Some(lens::LensState::Idle)
						},
						PathItem::End => {
							let event = lens::LensMoveEvent::Finished;
							events.push((pos, Box::new(event)));
							Some(lens::LensState::Idle)
						}
					};
					
					// Rebuild the path (even if it didn't change)
					lens.path_str = self.nodes.get_path_as_string(&lens.path)
						.expect("Failed to resolve path for lens.");
					
					new_state
				}
				
				_ => None
			};
			
			
			if let Some(new_state) = new_state {
				lens.state = new_state;
			}
		}
		
		while let Some((pos, mut event)) = events.pop() {
			self.fire_event_at_lens_id(
				pos,
				(*event).borrow_mut()
			);
		}
		
		// Remove all lenses that want to self-destruct.
		self.lenses.lenses.retain(
			|lens| lens.state != lens::LensState::Destruction
		);
		
		return self.lenses.lenses.is_empty()
	}
}

// Router path handling
impl Router {
	
	/// Resolves the next step towards a node from a path,
	/// a mutable offset into the path and the current node path.
	fn path_next(nodes: &node::RouterNodes,
	             dst_path: &str,
	             dst_off: &mut usize,
	             src_path: &[usize]
	) -> PathItem {
		// Parsing of root location only happens when `offset = 0`
		if *dst_off == 0 {
			if dst_path.starts_with("/") {
				// Bubbling
				if ! src_path.is_empty() {
					return PathItem::ToSuper;
				}
				
				*dst_off += 1;
				return PathItem::ToRoot
			}
			
			if dst_path.starts_with("./") {
				*dst_off += 2;
				return PathItem::ToSelf
			}
			
			if dst_path.starts_with("../") {
				*dst_off += 3;
				return PathItem::ToSuper
			}
		}
		
		// Slice away everything before the offset
		let mut path = dst_path.split_at(*dst_off).1;
		
		// Have we already reached the end?
		if path.len() == 0 {
			return PathItem::End;
		}
		
		// Slice away unnecessary slashes
		while path.starts_with("/") {
			*dst_off += 1;
			path = &path[1..];
		}
		
		if path.starts_with("./") {
			*dst_off += 2;
			return PathItem::ToSelf
		}
		
		if path.starts_with("../") {
			*dst_off += 3;
			return PathItem::ToSuper
		}
		
		let current = src_path.last();
		let current = match current {
			Some(x) => nodes.nodes.get(x),
			None => nodes.nodes.get(&0)
		};
		
		let current = match current {
			Some(x) => x,
			None => return PathItem::Error(format!("Could not resolve current.")),
		};
		
		let end = path.find("/")
			.unwrap_or(path.len());
		
		let name = &path[..end];
		
		// TODO: This part *should* be possible with iter()...?
		let mut next: Option<&node::Node> = None;
		for (_, node) in nodes.nodes.iter() {
			if ! node.is_named(name) {
				continue;
			}
			
			if ! node.is_child_of(current) {
				continue;
			}
			
			next = Some(node);
		}
		
		let next = match next {
			None => return PathItem::Error(format!("Could not find node: {}", name)),
			Some(x) => x,
		};
		
		*dst_off += end;
		return PathItem::ToNode(next.id);
	}
	
}

#[derive(Debug)]
pub enum PathItem {
	ToRoot, // `/`
	ToSelf, // `./`
	ToSuper, // `../`
	ToNode(usize), // `NAME`
	Error(String),
	End
}

impl std::fmt::Display for PathItem {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			PathItem::ToRoot => write!(fmt, "ToRoot"),
			PathItem::ToSelf => write!(fmt, "ToSelf"),
			PathItem::ToSuper => write!(fmt, "ToSuper"),
			PathItem::ToNode(x) => write!(fmt, "ToNode(#{})", *x),
			PathItem::Error(x) => write!(fmt, "Error({})", x),
			PathItem::End => write!(fmt, "End"),
		}
	}
}

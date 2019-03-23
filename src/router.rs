use core::borrow::{BorrowMut, Borrow};

/******************************************************************************/

pub struct Router {
	pub lenses: RouterLenses,
	pub nodes: RouterNodes,
}

/// Functions for building the router.
impl Router {
	
	pub fn new() -> Router {
		Router {
			lenses: RouterLenses::new(),
			nodes: RouterNodes::new(),
		}
	}
	
	pub fn new_lens(&mut self, name: &str, constructor: &Fn(&mut Lens)) {
		let mut lens = Lens {
			name: name.to_string(),
			path: vec![],
			state: LensState::Idle,
		};
		
		constructor(&mut lens);
		self.lenses.lenses.push(lens);
		self.lenses.handlers.push(Box::new(NULL_HANDLER));
	}
	
	pub fn new_node(&mut self, name: &str, parent: Option<usize>, constructor: &Fn(&mut Node)) {
		let parent = parent.or(Some(0));
		
		let id: Option<usize> = None;
		
		let id = id.unwrap_or(self.nodes.nodes.len());
		
		let mut node = Node {
			id: id,
			parent: parent,
			name: name.to_string(),
		};
		
		constructor(&mut node);
		
		self.nodes.nodes.push(Some(node));
	}
}

// Router update handling
impl Router {
	pub fn update(&mut self) -> bool {
		let mut events: Vec<(usize, Box<Event>)> = vec![];
		
		for (pos, lens) in self.lenses.lenses.iter_mut().enumerate() {
			
			if lens.path.is_empty() {
				// All lenses must be at least at root-level
				lens.state = LensState::Moving("/".to_string(), 0);
			}
			
			// Ignore all idle and destroying lenses
			if lens.state == LensState::Idle || lens.state == LensState::Destruction {
				continue
			}
			
			let new_state = match lens.state.borrow_mut() {
				LensState::Moving(path, offset) => {
					
					let step = Router::path_next(
						&self.nodes.nodes,
						path,
						offset,
						&lens.path
					);
					
					match step {
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
						PathItem::Error(e) => {
							let event = LensMoveEvent::Aborted;
							events.push((pos, Box::new(event)));
							Some(LensState::Idle)
						},
						PathItem::End => {
							let event = LensMoveEvent::Finished;
							events.push((pos, Box::new(event)));
							Some(LensState::Idle)
						}
					}
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
			|lens| lens.state != LensState::Destruction
		);
		
		return self.lenses.lenses.is_empty()
	}
}

// Router path handling
impl Router {
	
	/// Resolves the next step towards a node from a path,
	/// a mutable offset into the path and the current node path.
	fn path_next(nodes: &Vec<Option<Node>>,
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
			Some(x) => &nodes[*x],
			None => &nodes[0]
		};
		
		let current = match current {
			Some(x) => x,
			None => return PathItem::Error(format!("Could not resolve current.")),
		};
		
		let end = path.find("/")
			.unwrap_or(path.len());
		
		let name = &path[..end];
		
		// TODO: This part *should* be possible with iter()...?
		let mut next: Option<&Node> = None;
		for node in nodes.iter() {
			next = match node {
				Some(x) => {
					if ! x.is_named(name) {
						continue;
					}
					
					if ! x.is_child_of(current) {
						continue;
					}
					
					Some(x)
				},
				None => None
			};
			
			if let Some(_) = next {
				break;
			}
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

/******************************************************************************/

pub struct Node {
	pub parent: Option<usize>,
	pub name: String,
	pub id: usize,
}

impl Node {
	pub fn on_event(&mut self, event: &mut EventWrapper) {
		// TODO: Walk trough event-listeners/components...
	}
	
	pub fn is_child_of(&self, parent: &Node) -> bool {
		self.parent.map_or(false, |id| id == parent.id)
	}
	
	pub fn is_named(&self, name: &str) -> bool {
		self.name == name
	}
}

pub struct RouterNodes {
	nodes: Vec<Option<Node>>
}

impl RouterNodes {
	
	pub fn new() -> RouterNodes{
		let root_node = Some(Node {
			id: 0,
			parent: None,
			name: "root".to_string(),
		});
		
		RouterNodes {
			nodes: vec![root_node]
		}
	}
	
	pub fn get_mut_node_by_id(&mut self, id: usize) -> Option<&mut Node> {
		let node = self.nodes.get_mut(id);
		
		match node {
			None => return None,
			Some(node) => {
				match node {
					None => return None,
					Some(node) => {
						return Some(node)
					}
				}
			}
		}
	}
	
	pub fn get_node_by_id(&self, id: usize) -> &Option<Node> {
		self.nodes.get(id).unwrap_or(None.borrow())
	}
	
	pub fn get_node_id(&self, name: &str) -> Option<usize> {
		for (pos, node) in self.nodes.iter().enumerate() {
			if let Some(node) = node {
				if node.name == name {
					return Some(pos)
				}
			}
		}
		
		None
	}
}

/******************************************************************************/

pub struct Lens {
	pub name: String,
	pub path: Vec<usize>,
	pub state: LensState,
}

pub struct RouterLenses {
	lenses: Vec<Lens>,
	handlers: Vec<Box<LensHandler>>,
}

impl RouterLenses {
	
	pub fn new() -> RouterLenses {
		RouterLenses {
			lenses: vec![],
			handlers: vec![],
		}
	}
	
	pub fn get_mut_lens_by_id(&mut self, id: usize) -> Option<&mut Lens> {
		return self.lenses.get_mut(id)
	}
}

pub trait LensHandler {
	/* Called when the lens receives an event. */
	fn on_event(&mut self, event: &mut EventWrapper, lens: &Lens) -> LensState;
}

/* // TODO: Correctly implement this once https://areweasyncyet.rs/ is ready.
impl LensHandler {
	pub fn from_fn<'a, FnType: 'a>(handler_fn: FnType) -> Box<LensHandler + 'a>
		where FnType: Fn(&mut LensHandler, &mut EventWrapper) -> LensAction
	{
		struct FnLensHandler<'a> {
			handler_fn: Box<Fn(&mut LensHandler, &mut EventWrapper) -> LensAction + 'a>
		}
		
		impl<'a> LensHandler for FnLensHandler<'a> {
			fn on_event(&mut self, event: &mut EventWrapper) -> LensAction {
				(self.handler_fn)(self, event)
			}
		}
		
		Box::new(FnLensHandler {
			handler_fn: Box::new(handler_fn)
		})
	}
}
*/

const NULL_HANDLER: NullLensHandler = NullLensHandler {};

pub struct NullLensHandler {}

impl LensHandler for NullLensHandler {
	fn on_event(&mut self, _event: &mut EventWrapper, lens: &Lens) -> LensState {
		LensState::Idle
	}
}

#[derive(Clone)]
pub enum LensState {
	Idle,
	Moving(String, usize),
	Destruction,
}

impl PartialEq for LensState {
	fn eq(&self, other: &LensState) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}

enum LensMoveEvent {
	Finished,
	Aborted
}

impl Event for LensMoveEvent {
	fn is_passive(&self) -> bool { false }
}

/******************************************************************************/

/// A event that can be sent trough the router towards various destinations.
/// At the moment the only possible destination is a Lens.
pub trait Event: mopa::Any {
	///	If an event is passive, it can be fired at its destination
	///	regardless of what state the lens is in.
	fn is_passive(&self) -> bool;
}

// This is 100% necessary until `std::` provides Any for object-traits.
mopafy!(Event);

pub enum EventPhase {
	/// The event is being wrapped in a `EventWrapper`.
	Creation,
	
	/// The event is flowing towards its destination.
	Propagation,
	
	/// The event is being evaluated by its destination.
	Action,
	
	/// The event is flowing back towards its source.
	Bubbling
}

/// Wraps an event as it is processed by the [Router].
pub struct EventWrapper<'a> {
	#[allow(dead_code)]
	/// The event being processed.
	pub event: &'a mut Event,
	
	// --- State for the event
	phase: EventPhase,
	
	/// Can the event flow towards its destination?
	can_propagate: bool,
	
	/// Can the event be evaluated by its destination?
	can_default: bool,
	
	/// Can the event flow back towards its source?
	can_bubble: bool,
}

impl<'a> EventWrapper<'a> {
	/// Prevents the event from being evaluated by its destination.
	pub fn prevent_default(&mut self) {
		self.can_default = false;
	}
	
	/// Stops the flow of the event toward its destination.
	pub fn stop_propagation(&mut self) {
		self.can_propagate = false;
	}
	
	/// Stops the flow of the event back towards its source.
	pub fn stop_bubbling(&mut self) {
		self.can_bubble = false;
	}
}

/// Implementation details for event handling.
impl Router {
	/// Fires a single `Event` at a single `Lens`.
	pub fn fire_event_at_lens(&mut self, target: &str, event: &mut Event) {
		let lens_id = self.lenses.lenses.iter().position(|lens| { lens.name == target });
		let lens_id = match lens_id {
			Some(x) => x,
			None => return
		};
		
		self.fire_event_at_lens_id(lens_id, event);
	}
	
	/// Actual implementation for `fire_event_at_lens`.
	fn fire_event_at_lens_id(&mut self, lens_id: usize, event: &mut Event) {
		let lens = self.lenses.lenses.get_mut(lens_id);
		
		let lens = match lens {
			Some(x) => x,
			None => return
		};
		
		let lens_handler = self.lenses.handlers.get_mut(lens_id);
		let lens_handler = match lens_handler {
			Some(x) => x,
			None => return
		};
		
		// A lens can only receive an event if inactive or the event is PASSIVE.
		if !event.is_passive() {
			if lens.state != LensState::Idle {
				return
			}
		}
		
		// A lens without path cannot receive events
		/*
		if lens.path.len() == 0 {
			return;
		}
		*/
		
		let mut event_wrapper = EventWrapper {
			event,
			
			// Initial State
			phase: EventPhase::Creation,
			can_propagate: true,
			can_default: true,
			can_bubble: true,
		};
		
		event_wrapper.phase = EventPhase::Propagation;
		for node_id in lens.path.iter() {
			self.nodes.get_mut_node_by_id(*node_id).map(|n|
				n.on_event(&mut event_wrapper)
			);
			
			if !event_wrapper.can_propagate {
				break;
			}
		}
		
		let new_state = if event_wrapper.can_default {
			event_wrapper.phase = EventPhase::Action;
			
			(*lens_handler).on_event(&mut event_wrapper, lens)
		} else {
			LensState::Idle
		};
		
		if event_wrapper.can_bubble {
			event_wrapper.phase = EventPhase::Bubbling;
			for node_id in lens.path.iter().rev() {
				self.nodes.get_mut_node_by_id(*node_id).map(|n|
					n.on_event(&mut event_wrapper)
				);
				
				if !event_wrapper.can_bubble {
					break;
				}
			}
		}
		
		if lens.state != LensState::Idle {
			// Do start a new action if one is already running
			return
		}
		
		// Swap in the action, kicking off whatever action the lens wants...
		lens.state = new_state
	}
}

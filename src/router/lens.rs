use super::event;
use super::node;
use super::comp;
use std::any::TypeId;

/// Represents a movable pointer into the routing-tree.
pub struct Lens {
	/// The unique name of this lens.
	pub name: String,
	
	/// The current location of this lens in the routing-tree.
	pub path: Vec<usize>,
	
	/// The current location of this lens in the routing-tree, as a string.
	pub path_str: String,
	
	/// The state of the lens.
	pub state: State,
}

/// The owner of all lenses and their handlers.
pub struct Lenses {
	/// Collection of all lenses residing in the routing-tree.
	pub lenses: Vec<Lens>,
	
	/// Collection of handlers for individual lenses.
	pub handlers: Vec<Box<Handler>>,
}

impl Lenses {
	/// Creates a new empty collection of lenses.
	pub fn new() -> Lenses {
		Lenses {
			lenses: vec![],
			handlers: vec![],
		}
	}
	
	/// Returns a reference to a lens given its internal ID.
	pub fn get_lens_by_id(&mut self, id: usize) -> Option<&Lens> {
		return self.lenses.get(id)
	}
	
	/// Returns a mutable reference to a lens given its internal ID.
	pub fn get_mut_lens_by_id(&mut self, id: usize) -> Option<&mut Lens> {
		return self.lenses.get_mut(id)
	}
	
	/// Returns a reference to a lens given its name.
	pub fn get_lens_by_name(&mut self, name: &str) -> Option<&Lens> {
		return self.lenses.iter().find(|l| l.name == name)
	}
	
	/// Returns a mutable reference to a lens given its name.
	pub fn get_mut_lens_by_name(&mut self, name: &str) -> Option<&mut Lens> {
		return self.lenses.iter_mut().find(|l| l.name == name)
	}
}





/// A lens-handler is effectively the 'brain' of a lens.
/// All user-logic and -state for a lens is owned by the handler.
pub trait Handler {
	/// Called when the lens receives an event.
	/// Can return a new state for the lens.
	fn on_event<'a>(
		&mut self,
		event: &mut event::Wrapper,
		context: &mut Context
	) -> State;
}

/* // TODO: Correctly implement this once https://areweasyncyet.rs/ is ready.
impl LensHandler {
	/// Creates a new lens from an asynchronous function.
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

/// The only `NullHandler` ever needed, as a singleton.
pub const NULL_HANDLER: NullHandler = NullHandler {};

/// This is a lens-handler that doesn't do anything, ignoring all events.
pub struct NullHandler {}
impl Handler for NullHandler {
	fn on_event<'a>(
		&mut self,
		_event: &mut event::Wrapper,
		_context: &mut Context
	) -> State {
		State::Idle
	}
}





pub struct Context<'a> {
	pub lens: &'a Lens,
	pub nodes: &'a mut node::Nodes,
}

impl<'a> Context<'a> {
	pub fn new<'b>(lens: &'b Lens, nodes: &'b mut node::Nodes) -> Context<'b> {
		Context {lens, nodes}
	}
	
	pub fn get_lens_name(&self) -> &str {
		self.lens.name.as_str()
	}
	
	pub fn get_lens_state(&self) -> &State {
		&self.lens.state
	}
	
	pub fn get_lensed_node_id(&self) -> Option<usize> {
		match self.lens.path.last() {
			Some(node_id) => Some(*node_id),
			None => None
		}
	}
	
	pub fn get_lensed_node(&self) -> Option<&node::Node> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_node_by_id(node_id),
			None => None
		}
	}
	
	pub fn get_lensed_node_name(&self) -> Option<&str> {
		match self.get_lensed_node_id() {
			Some(node_id) => match self.nodes.get_node_by_id(node_id) {
				Some(node) => Some(node.name.as_str()),
				None => None
			},
			None => None
		}
	}
	
	pub fn get_path(&self) -> &[usize] {
		self.lens.path.as_slice()
	}
	
	pub fn get_path_str(&self) -> &str {
		self.lens.path_str.as_str()
	}
	
	pub fn get_component(&self, component_type_id: TypeId) -> Option<&comp::Component> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_node_component(node_id, component_type_id),
			None => None
		}
	}
	
	pub fn get_mut_component(&mut self, component_type_id: TypeId) -> Option<&mut comp::Component> {
		match self.get_lensed_node_id() {
			Some(node_id) => self.nodes.get_mut_node_component(node_id, component_type_id),
			None => None
		}
	}
	
	pub fn fire_event(&mut self) {
		// TODO: Implement event handling for the context.
		// Possibly by splitting it out from the router itself?
	}
}






/// The state of a lens within the router structure.
#[derive(Clone)]
pub enum State {
	/// The lens is idling at a node, doing its thing.
	Idle,
	
	/// The lens is moving around the router towards another node.
	Moving(String, usize),
	
	/// The lens is requesting that it'd be destroyed as soon as possible.
	Destruction,
}

impl PartialEq for State {
	/// Partial equality for the state of a lens, using the `LensState` discriminant.
	fn eq(&self, other: &State) -> bool {
		std::mem::discriminant(self) == std::mem::discriminant(other)
	}
}





/// Event that is fired repeatedly while a lens moves.
pub enum MoveEvent {
	/// Fired when a lens enters a node.
	EnterNode,
	/// Fired when a lens leaves a node.
	LeaveNode,
}

impl event::Event for MoveEvent {
	fn is_passive(&self) -> bool { false }
}

/// Event that is fired when a lens finishes moving.
pub enum MoveCompletionEvent {
	/// The lens successfully reached its destination.
	Finished,
	
	/// The lens failed to reach its destination.
	Aborted
}

impl event::Event for MoveCompletionEvent {
	fn is_passive(&self) -> bool { false }
}

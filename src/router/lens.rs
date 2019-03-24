use super::event;

pub struct Lens {
	pub name: String,
	pub path_str: String,
	pub state: State,
	pub path: Vec<usize>,
}

pub struct Lenses {
	pub lenses: Vec<Lens>,
	pub handlers: Vec<Box<Handler>>,
}

impl Lenses {
	
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

pub trait Handler {
	/// Called when the lens receives an event.
	/// Can return a new state for the lens.
	fn on_event(&mut self, event: &mut event::Wrapper, lens: &Lens) -> State;
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

/// This is a lens-handler that doesn't do anything, ignoring all events.
pub const NULL_HANDLER: NullHandler = NullHandler {};
pub struct NullHandler {}
impl Handler for NullHandler {
	fn on_event(&mut self, _event: &mut event::Wrapper, _lens: &Lens) -> State {
		State::Idle
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

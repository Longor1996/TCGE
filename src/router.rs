use core::borrow::{BorrowMut, Borrow};

pub fn new_router() -> Router {
    let router = Router {
        lenses: vec![],
        nodes: vec![],
    };
    
    return router;
}

/******************************************************************************/

pub struct Router {
    lenses: Vec<Lens>,
	nodes: Vec<Option<Node>>,
}

impl Router {
    //
    
    pub fn new_lens(&mut self, name: &str, constructor: &Fn(&mut Lens)) {
	    let mut lens = Lens {
		    name: name.to_string(),
		    path: vec![],
		    handler: Box::new(NullLensHandler {}),
		    action: LensAction::None,
	    };
	    
	    constructor(&mut lens);
	    self.lenses.push(lens);
    }
    
    pub fn update(&mut self) -> bool {
	    let mut events: Vec<(usize, Box<Event>)> = vec![];
	    
	    for (pos, lens) in self.lenses.iter_mut().enumerate() {
		
		    // Find all lenses that have an action running
		    if lens.action == LensAction::None {
			    continue
		    }
		    
		    // TODO: Actually implement routing...
		    // let mut finish_event = LensMoveEvent::Finished;
		    // events.push((pos, Box::new(finish_event)));
		    
		    lens.action = LensAction::None;
	    }
	    
	    while let Some((pos, mut event)) = events.pop() {
		    self.fire_event_at_lens_id(
			    pos,
			    (*event).borrow_mut()
		    );
	    }
	    
	    ;false
    }
	
	pub fn fire_event_at_lens(&mut self, target: &str, event: &mut Event) {
		let lens_id = self.lenses.iter().position(|lens| {lens.name == target});
		let lens_id = match lens_id {
			Some(x) => x,
			None => return
		};
		
		self.fire_event_at_lens_id(lens_id, event);
	}
    
    fn fire_event_at_lens_id(&mut self, target_id: usize, event: &mut Event) {
        let lens = self.lenses.get_mut(target_id);
        let lens = match lens {
            Some(x) => x,
            None => return
        };
	    
	    // A lens can only receive an event if inactive or the event is PASSIVE.
	    if ! event.is_passive() {
		    if lens.action != LensAction::None {
			    return
		    }
	    }
	    
	    if lens.path.len() == 0 {
		    return;
	    }
        
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
		    self.nodes[*node_id].as_mut().map(|n|
			    n.on_event(&mut event_wrapper)
		    );
			
		    if ! event_wrapper.can_propagate {
			    break;
		    }
	    }
	    
	    let action = if event_wrapper.can_default {
		    event_wrapper.phase = EventPhase::Action;
		    (*lens.handler).on_event(&mut event_wrapper)
	    } else {
		    LensAction::None
	    };
	    
	    if event_wrapper.can_bubble {
		    event_wrapper.phase = EventPhase::Bubbling;
		    for node_id in lens.path.iter().rev() {
			    self.nodes[*node_id].as_mut().map(|n|
				    n.on_event(&mut event_wrapper)
			    );
				
			    if ! event_wrapper.can_bubble {
				    break;
			    }
		    }
	    }
		
	    if lens.action != LensAction::None {
		    // Do start a new action if one is already running
		    return
	    }
	    
	    // Swap in the action, kicking off whatever action the lens wants...
	    lens.action = action
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
}

/******************************************************************************/

pub struct Lens {
    pub name: String,
    pub path: Vec<usize>,
    pub handler: Box<LensHandler>,
	pub action: LensAction,
}

pub trait LensHandler {
    /* Called when the lens receives an event. */
    fn on_event(&mut self, event: &mut EventWrapper) -> LensAction;
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

pub struct NullLensHandler {}
impl LensHandler for NullLensHandler {
	fn on_event(&mut self, event: &mut EventWrapper) -> LensAction {
		LensAction::None
	}
}

#[derive(PartialEq)]
pub enum LensAction {
    Move(String),
    None
}

enum LensMoveEvent {
	Finished, Aborted
}
impl Event for LensMoveEvent {
	fn is_passive(&self) -> bool {false}
}

/******************************************************************************/

pub trait Event {
	fn is_passive(&self) -> bool;
}
pub enum EventPhase {
	Creation, Propagation, Action, Bubbling
}
pub struct EventWrapper<'a> {
    #[allow(dead_code)]
	event: &'a mut Event,
	
	// State
	phase: EventPhase,
	can_propagate: bool,
	can_default: bool,
	can_bubble: bool,
}
impl<'a> EventWrapper<'a> {
	pub fn prevent_default(&mut self) {
		self.can_default = false;
	}
	pub fn stop_propagation(&mut self) {self.can_propagate = false; }
}

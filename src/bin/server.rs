extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate tcge;
use tcge::resources::Resources;
use tcge::router;

fn main() {
    let _res = Resources::from_exe_path().unwrap();
    
    let mut router = router::Router::new();
    
    let child_a_id = router.new_node("child-A", None, &|_|{});
    router.new_node("child-B", None, &|_|{});
    router.new_node("child-1", Some(child_a_id), &|_|{});
    
    router.new_lens("server", &|_| {
        println!("Server Lens Init");
        return Some(Box::new(ServerLens {
            counter: 0
        }));
    });
    
    println!("Loop Start");
    loop {
        if (&mut router).update() {
            println!("Loop Stop");
            break;
        }
        
        router.fire_event_at_lens("server", &mut Ping {});
    }
	
    println!("Goodbye!");
}

struct ServerLens {
    counter: usize
}
impl router::lens::Handler for ServerLens {
    fn on_event(
        &mut self,
        event: &mut router::event::Wrapper,
        lens: &router::lens::Lens,
        _nodes: &mut router::node::Nodes
    ) -> router::lens::State {
        self.counter += 1;
        if self.counter > 10 {
            return router::lens::State::Destruction
        }
        
        // Downcasting by using MOPA::Any
        event.event.downcast_ref::<Ping>().map(|_| {
            println!("PONG!");
        });
        
        println!("Received event: {} @ {}", self.counter, lens.path_str);
        router::lens::State::Idle
    }
}

struct Ping {}
impl router::event::Event for Ping {
    fn is_passive(&self) -> bool {true}
}

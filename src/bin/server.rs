extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate tcge;
use tcge::resources::Resources;
use tcge::router;

fn main() {
    let res = Resources::from_exe_path().unwrap();
    
    /*
    use tcge::util::utf8;
    use tcge::util::cdml;
    println!("Hello, Server! {}", tcge::MAGIC);
    
    let reader = res.open_stream("test.cdml").unwrap();
    let mut reader= utf8::UTF8Read::new(reader);
    
    println!();
    while let Ok(x) = reader.read() {
        print!("{}", x)
    }
    println!();
    
    // let mut cdmlReader = cdml::CDMLReader::read(reader);
    */
    
    let mut router = router::Router::new();
    
    router.new_node("child-A", None, &|_|{});
    router.new_node("child-B", None, &|_|{});
    
    let child_a_id = router.nodes.get_node_id("child-A");
    router.new_node("child-1", child_a_id, &|_|{});
    
    router.new_lens("server", &|lens| {
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
impl router::LensHandler for ServerLens {
    fn on_event(&mut self, event: &mut router::EventWrapper, lens: &router::Lens) -> router::LensState {
        self.counter += 1;
        if self.counter > 10 {
            return router::LensState::Destruction
        }
        
        // Downcasting by using MOPA::Any
        event.event.downcast_ref::<Ping>().map(|e| {
            println!("PONG!");
        });
        
        println!("Received event: {}", self.counter);
        router::LensState::Idle
    }
}

struct Ping {}
impl router::Event for Ping {
    fn is_passive(&self) -> bool {true}
}

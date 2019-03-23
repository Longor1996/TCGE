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
    
    let childA_id = router.get_node_id("child-A");
    router.new_node("child-1", childA_id, &|_|{});
    
    if true || true {
        let dst_path = "/child-A/child-1";
        let mut dst_off = 0;
        let mut src_path = vec![];
        
        loop {
            let step = router.path_next(dst_path, &mut dst_off, &src_path);
            println!("--- STEP: {}", step);
            
            match step {
                router::PathItem::ToSelf => {continue;},
                router::PathItem::ToRoot => {&src_path.clear();},
                router::PathItem::ToSuper => {&src_path.pop();},
                router::PathItem::ToNode(x) => {&src_path.push(x);},
                router::PathItem::Error(_) => {break;},
                router::PathItem::End => {break;},
            };
        }
        
        return;
    }
    
    router.new_lens("server", &|lens| {
        println!("Server Lens Init");
        lens.handler = Box::new(ServerLens {
            counter: 0
        });
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
    fn on_event(&mut self, event: &mut router::EventWrapper) -> router::LensState {
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

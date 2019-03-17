extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate tcge;
use tcge::resources::Resources;
use tcge::util::utf8;
use tcge::router;
//use tcge::util::cdml;

fn main() {
    let res = Resources::from_exe_path().unwrap();
    
    /*
    println!("Hello, Server! {}", tcge::MAGIC);
    
    let reader = res.open_stream("test.cdml").unwrap();
    let mut reader= utf8::UTF8Read::new(reader);
    
    println!();
    while let Ok(x) = reader.read() {
        print!("{}", x)
    }
    println!();
    */
    
    // let mut cdmlReader = cdml::CDMLReader::read(reader);
    
    let mut router = router::new_router();
    
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
        
        let mut ping = Ping {};
        router.fire_event_at_lens("server", &mut ping);
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
        
        println!("Received event: {}", self.counter);
        router::LensState::Idle
    }
}

struct Ping {}
impl router::Event for Ping {
    fn is_passive(&self) -> bool {true}
}

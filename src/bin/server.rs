#[macro_use]
extern crate log;
extern crate simplelog;

extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate tcge;
use tcge::resources::Resources;
use tcge::router;

fn main() {
    use simplelog::*;
    use std::fs::File;
    let current_exe = std::env::current_exe().expect("Failed to get path of the 'server' executable.");
    let current_dir = current_exe.parent().expect("Failed to get path of the 'server' executables parent directory.");
    let log_file = current_dir.join("server.log");
    
    let mut log_config = Config::default();
    log_config.time_format = Some("[%Y-%m-%d %H:%M:%S]");
    
    println!("[HINT] Log file location: {}", log_file.to_str().unwrap_or("ERROR"));
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Trace, log_config).expect("Failed to set up TermLogger for server."),
            WriteLogger::new(LevelFilter::Info, log_config, File::create(log_file).expect("Failed to set up FileLogger for server.")),
        ]
    ).unwrap();
    info!("Server startup...");
    
    let _res = Resources::from_exe_path().expect("Failed to setup root resource provider for server.");
    
    let mut router = router::Router::new();
    
    let child_a_id = router.new_node("child-A", None, &|_|{});
    router.new_node("child-B", None, &|_|{});
    router.new_node("child-1", Some(child_a_id), &|_|{});
    
    router.new_lens("server", &|_| {
        info!("Server Lens Init");
        return Some(Box::new(ServerLens {
            counter: 0
        }));
    });
    
    info!("Loop Start");
    loop {
        if (&mut router).update() {
            info!("Loop Stop");
            break;
        }
        
        router.fire_event_at_lens("server", &mut Ping {});
    }
    
    info!("Server shutdown!");
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
            info!("Received PONG!");
        });
    
        info!("Received event: {} @ {}", self.counter, lens.path_str);
        router::lens::State::Idle
    }
}

struct Ping {}
impl router::event::Event for Ping {
    fn is_passive(&self) -> bool {false}
}

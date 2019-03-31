#[macro_use]
extern crate log;
extern crate simplelog;

extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate tcge;
use tcge::resources::Resources;

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
    
    info!("Server shutdown!");
}

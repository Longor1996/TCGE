#[macro_use] extern crate failure;
extern crate TCGE;

use TCGE::resources::Resources;
use TCGE::util::utf8;
use TCGE::util::cdml;

fn main() {
    let res = Resources::from_exe_path().unwrap();

    println!("Hello, Server! {}", TCGE::MAGIC);

    let mut reader = res.open("test.cdml").unwrap();
    let mut reader= utf8::UTF8Read::new(reader);

    print!("\n");
    loop {
        match reader.read() {
            Ok(x) => print!("{}", x),
            Err(e) => break
        }
    }
    print!("\n");

    // let mut cdmlReader = cdml::CDMLReader::read(reader);

}

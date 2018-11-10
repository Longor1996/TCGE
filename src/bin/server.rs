#[macro_use]
extern crate failure;
extern crate TCGE;

use TCGE::resources::Resources;
use TCGE::util::utf8;
use TCGE::util::cdml;

fn main() {
    let res = Resources::from_exe_path().unwrap();

    println!("Hello, Server! {}", TCGE::MAGIC);

    let reader = res.open_stream("test.cdml").unwrap();
    let mut reader= utf8::UTF8Read::new(reader);

    println!();
    while let Ok(x) = reader.read() {
        print!("{}", x)
    }
    println!();

    // let mut cdmlReader = cdml::CDMLReader::read(reader);

}

extern crate failure;
#[allow(unused_imports)]
use failure::Fail;

extern crate tcge;
use tcge::resources::Resources;
use tcge::util::utf8;
//use tcge::util::cdml;

fn main() {
    let res = Resources::from_exe_path().unwrap();

    println!("Hello, Server! {}", tcge::MAGIC);

    let reader = res.open_stream("test.cdml").unwrap();
    let mut reader= utf8::UTF8Read::new(reader);

    println!();
    while let Ok(x) = reader.read() {
        print!("{}", x)
    }
    println!();

    // let mut cdmlReader = cdml::CDMLReader::read(reader);

}

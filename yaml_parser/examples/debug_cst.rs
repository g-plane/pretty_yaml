use std::{env, fs};

fn main() {
    let file = fs::read_to_string(env::args().nth(1).unwrap()).unwrap();
    let tree = yaml_parser::parse(&file).unwrap();
    println!("{:#?}", tree);
}

use std::{env, fs};

fn main() {
    let file = fs::read_to_string(env::args().nth(1).unwrap()).unwrap();
    match yaml_parser::parse(&file) {
        Ok(tree) => println!("{tree:#?}"),
        Err(err) => eprintln!("{err}"),
    };
}

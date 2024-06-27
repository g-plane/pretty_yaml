use pretty_yaml::format_text;
use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = env::args().nth(1).unwrap();
    let input = fs::read_to_string(&file_path)?;
    let formatted = format_text(&input);
    print!("{formatted}");
    Ok(())
}

#![feature(negative_impls)]

use ron::de::from_str;
use serde::Deserialize;
use std::collections::HashMap;
mod parser;
mod eval;
mod util;
mod frontend;
fn main() {
    // first, get the localisation file from disk
    let manifest_path = Path::new("assets/en_uk.ron");
    let manifest_file = match File::open(manifest_path) {
        Err(reason) => {
            println!("{}", format!("Aborting: could not open file -- {}", reason).red());
            std::process::exit(1);
        }
        Ok(file) => file
    };
    let mut manifest_str = String::new();
    if let Err(reason) = manifest_file.read_to_string(&mut manifest_str) {
        println!("{}", format!("Aborting: could read file -- {}", reason).red());
        std::process::exit(1);
    }
    // parse it as a hashmap
    let manifest: HashMap<&str, String> = match from_str(&manifest_str) {
        Err(e) => {
            println!("{}", format!("Developer error: localisation file is bad -- {}", e).red());
            std::process::exit(1);
        }
        Ok(map) => map
    };

}

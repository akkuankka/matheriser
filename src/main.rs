#![feature(negative_impls)]
#![feature(drain_filter)]

mod eval;
mod frontend;
mod parser;
mod util;

use colored::Colorize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use ron::de::from_str;
use std::collections::HashMap;
use frontend::{CommandLine, Frontend};
use parser::parse_string;

use structopt::StructOpt;

enum FrontendOpt {
    CommandLine
}
impl std::str::FromStr for FrontendOpt {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "c" => FrontendOpt::CommandLine,
            _ => FrontendOpt::CommandLine
        })
    }
}

#[derive(StructOpt)]
#[structopt(name = "matheriser", about = "evaluates mathematical expressions of increasing complexity")]
struct Options {
    #[structopt(short, long, default_value = "c")]
    interface: FrontendOpt,

    #[structopt(short = "e", long = "immediate", long = "evaluate")]
    immediate: Option<String>,

    #[structopt(short, long, default_value = "en-uk")]
    language: String
}

fn crash() -> ! {
    std::process::exit(1);
}


fn main() {
    let opt = Options::from_args();
    let mut locbuffer = String::new();
    let localisation_map = get_localisation(&opt.language, &mut locbuffer);

    if let Some(expr) = &opt.immediate {
        let tree = match parse_string(expr) {
            Err(why) => {
                eprintln!("{}", format!("{}", why).red());
                crash()
            }
            Ok(tree) => tree
        };
        match tree.eval() {
            Err(why) => {
                eprintln!("{}", format!("{}", why).red());
                crash()
            }
            Ok(answer) => {
                println!("{}", format!("{}", answer).blue());
            }
        }
    }
    else {
        match opt.interface {
            FrontendOpt::CommandLine => {
                let mut frontend = CommandLine::new(&localisation_map);
                if let Err(e) = frontend.run() {
                    eprintln!("{}", format!("{}", e).red())
                }

            }
        }
    }
}





const LANGUAGES_MANIFEST: &'static str = include_str!("../assets/manifest.ron");
use directories::ProjectDirs;
use std::path::PathBuf;

fn get_localisation<'a>(query: &String, locbuffer: &'a mut String) -> HashMap<&'a str, String> {
    let manifest: Vec<&str> = match from_str(LANGUAGES_MANIFEST) {
        Err(reason) => {
            eprintln!("{}", format!("Aborting: could not parse the languages manifest -- {}", reason).red());
            crash()
        }
        Ok(list) => list
    };
    if manifest.contains(&&**query) {
        // let localisation_file_path = Path::new(&format!{"assets/{}.ron", query});
        let localisation_file_path = match seek_get_localisation_file() {
            Ok(p) => p,
            Err(reason) => {
                eprintln!(
                    "{}",
                    format!("Aborting: could not open file -- {}", reason).red()
                );
                std::process::exit(1);
            }
        };
        let mut localisation_file = match File::open(localisation_file_path.join(PathBuf::from(format!("{}.ron", query)))) {
            Err(reason) => {
                eprintln!(
                    "{}",
                    format!("Aborting: could not open file -- {}", reason).red()
                );
                std::process::exit(1);
            }
            Ok(file) => file,
        };
        if let Err(reason) = localisation_file.read_to_string(locbuffer) {
            eprintln!(
                "{}",
                format!("Aborting: could read file -- {}", reason).red()
            );
            std::process::exit(1);
        }
        // parse it as a hashmap
        let manifest: HashMap<&str, String> = match from_str(locbuffer) {
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("Developer error: localisation file is bad -- {}", e).red()
                );
                std::process::exit(1);
            }
            Ok(map) => map,
        };
        manifest
    }
    else {
        eprintln!("Aborting: language {} not found", query);
        crash()
    }
}

fn seek_get_localisation_file() -> Result<PathBuf, String> {
    let os = std::env::consts::OS; 
    let linux_systemwide_dir: PathBuf = PathBuf::from("/usr/share/matheriser/assets/");
    if os != "macos" && os != "windows" {
        if linux_systemwide_dir.exists() {
            return Ok(linux_systemwide_dir)
        }
    }
    if let Some(pd) = ProjectDirs::from("", "", "matheriser") {
        let path = pd.data_local_dir().join(PathBuf::from("assets"));
        if path.exists() {
            Ok(path)
        } else {
            Err("Error: Localisation path does not exist".to_string())
        }
    } else {
        Err("Error: Could not find the localisation directory".to_string())
    }

}


    // // first, get the localisation file from disk
    // let manifest_path = Path::new("assets/en-uk.ron");
    // let mut manifest_file = match File::open(manifest_path) {
    //     Err(reason) => {
    //         println!(
    //             "{}",
    //             format!("Aborting: could not open file -- {}", reason).red()
    //         );
    //         std::process::exit(1);
    //     }
    //     Ok(file) => file,
    // };
    // let mut manifest_str = String::new();
    // if let Err(reason) = manifest_file.read_to_string(&mut manifest_str) {
    //     println!(
    //         "{}",
    //         format!("Aborting: could read file -- {}", reason).red()
    //     );
    //     std::process::exit(1);
    // }
    // // parse it as a hashmap
    // let manifest: HashMap<&str, String> = match from_str(&manifest_str) {
    //     Err(e) => {
    //         println!(
    //             "{}",
    //             format!("Developer error: localisation file is bad -- {}", e).red()
    //         );
    //         std::process::exit(1);
    //     }
    //     Ok(map) => map,
    // };
    // let mut frontend = CommandLine::new(&manifest);
    // if let Err(reason) = frontend.run() {
    //     println!("{}", reason);
    //     std::process::exit(1);
    // };

    // println!("{}", "goodbye!".green())


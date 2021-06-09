/*! This is the commandline frontend for matheriser, which waits for input and is singlethreaded, doesn't automagically change the expressions you enter */

use std::collections::HashMap;

/// The information struct for the frontend
pub struct CommandLine<'m, 'k: 'm> {
    preamble: bool,
    manifest: &'m HashMap<&'k str, String>,
}
impl<'m, 'k> CommandLine<'m, 'k> {
    const PROMPT_TEXT: &'static str = "matherise";

    pub fn new(manifest: &'m HashMap<&'k str, String>) -> Self {
        CommandLine {
            preamble: true,
            manifest: manifest,
        }
    }
}

use crate::parser::parse_string;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
impl<'m, 'k> super::Frontend for CommandLine<'_, '_> {
    fn run(&mut self) -> Result<(), String> {
        //println!("began to run");
        if self.preamble {
            let coloured_preamble = self
                .manifest
                .get("preamble") // get the preamble
                .ok_or("couldn't get the preamble")? // unwrap it
                .green();
            println!("{}", coloured_preamble)
            //println!("preambled");
        }
        loop {
            let input_result: Result<String, _> = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(CommandLine::PROMPT_TEXT)
                .interact_text();
            let input = match input_result {
                Ok(s) => s,
                Err(_) => {
                    return Err("Couldn't get your input".into());
                }
            };
            if input == "!qt" {
                break;
            }
            let out_text = parse_string(&input).and_then(|x| x.eval())?;
            println!("         >=> {}", out_text);
        }
        Ok(())
    }
}

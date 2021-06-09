/*! This is the commandline frontend for matheriser, which waits for input and is singlethreaded, doesn't automagically change the expressions you enter */

use std::collections::HashMap;

/// The information struct for the frontend
pub struct CommandLine<'m, 'k: 'm> {
    preamble: bool,
    manifest: &'m HashMap<&'k str, String>,
}
impl<'m, 'k> CommandLine<'m, 'k> {
    const PROMPT_TEXT: &'static [u8] = b"matherise >";

    pub fn new(manifest: &'m HashMap<&'k str, String>) -> Self {
        CommandLine {
            preamble: true,
            manifest: manifest,
        }
    }

    fn prompt(&self, f: &Stdout) {
        println!("prompting !");
        let mut handle = f.lock();
        let _= handle.write_all(Self::PROMPT_TEXT);
        let _= handle.flush();
    }

    fn input(&self, f: &Stdin) -> Result<String, &'static str> {
        let buffer = &mut String::with_capacity(10);
        match f.read_line(buffer) {
            Err(_) => return Err("Error: couldn't read from stdin"),
            Ok(_) => {}
        }
        Ok(buffer.trim().to_string())
    }
}

use crate::parser::parse_string;
use colored::Colorize;
use std::io::{self, Stdin, Stdout, Write};
#[allow(unused_must_use)]
impl<'m, 'k> super::Frontend for CommandLine<'_, '_> {
    fn run(&mut self) -> Result<(), &'static str> {
        //println!("began to run");
        let stdout = io::stdout();
        if self.preamble {
            stdout.lock().write_all(
                self.manifest
                    .get("preamble")
                    .ok_or("Developer error: preamble not found in language file")?
                    .blue()
                    .as_bytes(),
            );
            //println!("preambled");
        }
        let stdin = io::stdin();
        let mut locked_stdout = stdout.lock();
        loop {
        //    println!("looping");
            self.prompt(&stdout);
            let input = self.input(&stdin)?;
            if input == "!qt" {
                break;
            } else {
                match parse_string(&input) {
                    Err(e) => {
                        locked_stdout.write_all(e.red().as_bytes());
                    }
                    Ok(t) => match t.eval() {
                        Err(e) => {
                            locked_stdout.write_all(e.red().as_bytes());
                        }
                        Ok(value) => {
                            locked_stdout.write_all(format!("{}", value).blue().as_bytes());
                        }
                    },
                }
                locked_stdout.flush();
            }
        }
        Ok(())
    }
}

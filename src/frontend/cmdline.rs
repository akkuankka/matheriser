/*! This is the commandline frontend for matheriser, which waits for input and is singlethreaded, doesn't automagically change the expressions you enter */

use std::collections::HashMap;

/// The information struct for the frontend
pub struct CommandLine<'m, 'k: 'm> {
    preamble: bool,
    manifest: &'m HashMap<&'k str, String>,
}
impl<'m, 'k> CommandLine<'m, 'k> {
    const prompt_text: &'static [u8] = b"matherise >";

    fn new(manifest: &'m HashMap<&'k str, String>) -> Self {
        CommandLine {
            preamble: true,
            manifest: manifest,
        }
    }

    fn prompt(&self, f: &Stdout) {
        let mut handle = f.lock();
        handle.write_all(Self::prompt_text);
    }

    fn input(&self, f: &Stdin) -> Result<String, &'static str> {
        let mut buffer = &mut String::with_capacity(10);
        match f.read_line(buffer) {
            Err(e) => return Err("Error: couldn't read from stdin"),
            Ok(_) => {}
        }
        Ok(buffer.trim().to_string())
    }
}

use crate::parser::parse_string;
use colored::Colorize;
use std::io::{self, Stdin, Stdout, Write};

impl<'m, 'k> super::Frontend for CommandLine<'_, '_> {
    fn run(&mut self) -> Result<(), &'static str> {
        let stdout = io::stdout();
        if self.preamble {
            stdout.lock().write_all(
                self.manifest
                    .get("preamble")
                    .ok_or("Developer error: preamble not found in language file")?
                    .blue()
                    .as_bytes(),
            );
        }
        let stdin = io::stdin();
        loop {
            self.prompt(&stdout);
            let input = self.input(&stdin)?;
            if input == "!qt" {
                break;
            } else {
                match parse_string(&input) {
                    Err(e) => {
                        stdout.lock().write_all(e.red().as_bytes());
                    }
                    Ok(t) => match t.eval() {
                        Err(e) => {
                            stdout.lock().write_all(e.red().as_bytes());
                        }
                        Ok(e) => {
                            stdout.lock().write_all(format!("{}", e).blue().as_bytes());
                        }
                    },
                }
            }
        }
        Ok(())
    }
}

mod display;
mod cmdline;
pub use display::*;

pub use cmdline::CommandLine;

pub trait Frontend {
    fn run(&mut self) -> Result<(), String>;
}

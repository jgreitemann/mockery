use colored::Colorize;
use std::fmt::{Display, Formatter};

pub trait Termination {
    fn report(self) -> i32;
}

#[derive(Debug)]
pub struct CLIError(pub String);

impl<E: std::error::Error> From<E> for CLIError {
    fn from(e: E) -> Self {
        CLIError(e.to_string())
    }
}

impl Display for CLIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub type CLIResult<T> = std::result::Result<T, CLIError>;

impl Termination for CLIResult<()> {
    fn report(self) -> i32 {
        eprintln!("{} {}", "error:".red().bold(), &self.unwrap_err());
        1
    }
}

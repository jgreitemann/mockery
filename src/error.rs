use colored::Colorize;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub trait Termination {
    fn report(self) -> i32;
}

#[derive(Debug)]
pub enum CLIError {
    SourceFileNotFound(std::io::Error),
    SpecifiedCompilationDatabaseNotFound(std::io::Error),
    CompilationDatabaseSearchFailed,
    CompilationDatabaseSearchStartingPointNotFound(PathBuf, std::io::Error),
    CompileCommandNotFound(PathBuf),
    SourceError(clang::SourceError),
    InterfaceClassNotFound(String),
    NotYetImplemented,
}

impl Display for CLIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CLIError::SourceFileNotFound(io_err) => {
                f.write_fmt(format_args!(
                    "Failed to open source file: {}",
                    io_err.to_string().italic(),
                ))
            }
            CLIError::SpecifiedCompilationDatabaseNotFound(io_err) => {
                f.write_fmt(format_args!(
                    "Could not find the specified compile commands database: {}",
                    io_err.to_string().italic(),
                ))
            }
            CLIError::CompilationDatabaseSearchFailed => {
                f.write_str("Could not find compile commands database within the specified search radius")
            }
            CLIError::CompilationDatabaseSearchStartingPointNotFound(starting_point, io_err) => {
                f.write_fmt(format_args!(
                    "Could not find the starting point '{}' for the compile commands database search: {}",
                    starting_point.to_str().unwrap().yellow(),
                    io_err.to_string().italic(),
                ))
            }
            CLIError::CompileCommandNotFound(source_file) => f.write_fmt(format_args!(
                "Failed to find compile command for '{}' in database",
                source_file.to_str().unwrap().yellow(),
            )),
            CLIError::SourceError(source_err) => f.write_fmt(format_args!(
                "The source file could not be parsed: {}",
                source_err.to_string().italic(),
            )),
            CLIError::InterfaceClassNotFound(interface_name) => f.write_fmt(format_args!(
                "No interface class named `{}` was found in the specified translation unit",
                interface_name.yellow(),
            )),
            CLIError::NotYetImplemented => f.write_str("Not yet implemented"),
        }
    }
}

impl std::error::Error for CLIError {}

pub type CLIResult<T> = std::result::Result<T, CLIError>;

impl Termination for CLIResult<()> {
    fn report(self) -> i32 {
        eprintln!("{} {}", "error:".red().bold(), &self.unwrap_err());
        1
    }
}

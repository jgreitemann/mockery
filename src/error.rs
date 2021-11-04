#[derive(Debug)]
pub struct CLIError(pub String);

impl<E: std::error::Error> From<E> for CLIError {
    fn from(e: E) -> Self {
        CLIError(e.to_string())
    }
}

pub type CLIResult = Result<(), CLIError>;

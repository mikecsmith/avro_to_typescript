use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum GenerationError {
    Module(String),
    Utf8(std::string::FromUtf8Error),
    Io(std::io::Error),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::Module(e) => write!(f, "Module error: {}", e),
            GenerationError::Utf8(e) => write!(f, "UTF-8 error: {}", e),
            GenerationError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for GenerationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GenerationError::Module(_) => None,
            GenerationError::Utf8(e) => Some(e),
            GenerationError::Io(e) => Some(e),
        }
    }
}

impl From<std::string::FromUtf8Error> for GenerationError {
    fn from(err: std::string::FromUtf8Error) -> GenerationError {
        GenerationError::Utf8(err)
    }
}

impl From<String> for GenerationError {
    fn from(err: String) -> GenerationError {
        GenerationError::Module(err)
    }
}

impl From<std::io::Error> for GenerationError {
    fn from(err: std::io::Error) -> GenerationError {
        GenerationError::Io(err)
    }
}

//! Error types and conversion functions.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// An enum containing all kinds of game framework errors.
#[derive(Debug, Clone)]
pub enum GameError {
    /// An error in the filesystem layout
    FilesystemError(String),
    /// An error in the config file
    ConfigError(String),
    /// Happens when an `winit::EventsLoopProxy` attempts to
    /// wake up an `winit::EventsLoop` that no longer exists.
    EventLoopError(String),
    /// An error trying to load a resource, such as getting an invalid image file.
    ResourceLoadError(String),
    /// Unable to find a resource; the `Vec` is the paths it searched for and associated errors
    ResourceNotFound(String, Vec<(std::path::PathBuf, GameError)>),
    /// Something went wrong in the renderer
    RenderError(String),
    /// Something went wrong trying to read from a file
    IOError(Arc<std::io::Error>),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameError::ConfigError(ref s) => write!(f, "Config error: {}", s),
            GameError::ResourceLoadError(ref s) => write!(f, "Error loading resource: {}", s),
            GameError::ResourceNotFound(ref s, ref paths) => write!(
                f,
                "Resource not found: {}, searched in paths {:?}",
                s, paths
            ),
            GameError::WindowError(ref e) => write!(f, "Window creation error: {}", e),
            _ => write!(f, "GameError {:?}", self),
        }
    }
}

impl Error for GameError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            GameError::WindowCreationError(ref e) => Some(&**e),
            GameError::IOError(ref e) => Some(&**e),
            GameError::ShaderProgramError(ref e) => Some(e),
            _ => None,
        }
    }
}

pub type GameResult<T = ()> = Result<T, GameError>;

impl From<std::io::Error> for GameError {
    fn from(e: std::io::Error) -> GameError {
        GameError::IOError(Arc::new(e))
    }
}

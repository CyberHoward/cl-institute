use std::path::PathBuf;

use thiserror::Error;

/// Errors that can occur during file-system persistence operations.
#[derive(Debug, Error)]
pub enum StoreError {
    /// An I/O error occurred while reading or writing a file.
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse a TOML file.
    #[error("TOML parse error in {path}: {source}")]
    TomlParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// Failed to serialize a value to TOML.
    #[error("TOML serialization error for {path}: {source}")]
    TomlSerialize {
        path: PathBuf,
        source: toml::ser::Error,
    },

    /// Failed to parse a JSON file.
    #[error("JSON parse error in {path}: {source}")]
    JsonParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    /// Failed to serialize a value to JSON.
    #[error("JSON serialization error for {path}: {source}")]
    JsonSerialize {
        path: PathBuf,
        source: serde_json::Error,
    },

    /// A required file was not found.
    #[error("Required file not found: {0}")]
    FileNotFound(PathBuf),

    /// A required directory was not found.
    #[error("Required directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    /// The institution directory structure is invalid.
    #[error("Invalid project structure: {0}")]
    InvalidStructure(String),
}

impl StoreError {
    /// Create an I/O error with path context.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    /// Create a TOML parse error with path context.
    pub fn toml_parse(path: impl Into<PathBuf>, source: toml::de::Error) -> Self {
        Self::TomlParse {
            path: path.into(),
            source,
        }
    }

    /// Create a TOML serialization error with path context.
    pub fn toml_serialize(path: impl Into<PathBuf>, source: toml::ser::Error) -> Self {
        Self::TomlSerialize {
            path: path.into(),
            source,
        }
    }

    /// Create a JSON parse error with path context.
    pub fn json_parse(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::JsonParse {
            path: path.into(),
            source,
        }
    }

    /// Create a JSON serialization error with path context.
    pub fn json_serialize(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::JsonSerialize {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, StoreError>;

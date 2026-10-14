use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Io error")]
    IOError(String),
    #[error("Parse error")]
    ParseError,
    #[error("Net error")]
    NetError(String),
    #[error("Serialization/Deserialization error")]
    SerdeError(String),
    #[error("GridDB API error")]
    GridDBError(String)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError(value.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::SerdeError(value.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::SerdeError(value.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::NetError(value.to_string())
    }
}
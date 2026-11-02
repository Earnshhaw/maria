use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Io error occured: ({err_msg:?})")]
    IOError { err_msg: String },
    #[error("Parse error: ({err_msg:?})")]
    ParseError { err_msg: String },
    #[error("Net error: ({err_msg})")]
    NetError { err_msg: String },
    #[error("Serialization/Deserialization error: ({err_msg})")]
    SerdeError { err_msg: String },
    #[error("GridDB API error: ({err_msg})")]
    GridDBError { err_msg: String },
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IOError {
            err_msg: value.to_string(),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::SerdeError {
            err_msg: value.to_string(),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::SerdeError {
            err_msg: value.to_string(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::NetError {
            err_msg: value.to_string(),
        }
    }
}

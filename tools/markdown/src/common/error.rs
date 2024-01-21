use thiserror::Error;
use std;


#[derive(Error, Debug)]
pub enum RMarkodwnError {
    #[error("rmarkdwon IO error")]
    IOError(#[from] std::io::Error),
    
    #[error("rmarkdwon time error")]
    TimeError(#[from] std::time::SystemTimeError),
    
    #[error("rmakdwon ParseError. (line:{line:?}, detail:{detail:?}) ")]
    ParseError {
        line: usize,
        detail: String,
    },

    #[error("unknown data store error")]
    Unknown(String),
}




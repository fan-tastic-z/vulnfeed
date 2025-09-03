use error_stack::Report;

use crate::errors::Error;

pub mod cli;
pub mod config;
pub mod domain;
pub mod errors;
pub mod input;
pub mod output;
pub mod utils;

pub type AppResult<T> = Result<T, Report<Error>>;

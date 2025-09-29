use std::{
    env,
    path::PathBuf,
};

use crate::{
    Result,
    error::CliError,
};

pub fn get_path_or_curr_dir(path: Option<PathBuf>) -> Result<PathBuf>
{
    match path
    {
        None => env::current_dir().map_err(|_| CliError::AccessCurrentDirectory.into()),
        Some(path) => Ok(path),
    }
}

pub fn format_duration(miliseconds: u128) -> String
{
    if miliseconds < 1000
    {
        format!("{:.2}ms", miliseconds)
    }
    else
    {
        format!("{:.2}s", miliseconds as f64 / 1000.0)
    }
}

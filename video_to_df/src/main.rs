mod command;
mod config;
mod error;
mod monoframe;
mod output;
mod sdf;

use std::{
    env,
    error::Error,
    fmt::Result as FormatResult,
};

use ffmpeg_next as ffmpeg;

use crate::{
    command::Command,
    config::*,
    error::*,
    ffmpeg::Error as FFmpegError,
    monoframe::MonoFrame,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main()
{
    match run()
    {
        Ok(()) => (),
        Err(e) =>
        {
            eprint!("{}", e);
        },
    }
}

fn run() -> Result<()>
{
    // Straightforward
    let mut args = env::args().skip(1);

    // Pretty good, although the error is kinda weird (but valid)
    let command_str = args.next().ok_or(CliError::NoCommand)?;

    // Again, okay, except the error. This is an appropriate simplification. Commands must be kept
    // valid
    let command = Command::from_name(&command_str).ok_or(CliError::UnknownCommand(command_str))?;

    // uh oh
    command.execute(args)
}

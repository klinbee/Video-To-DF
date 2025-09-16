mod command;
mod config;
mod error;
mod monoframe;
mod output;
mod sdf;

use std::{
    env::{
        self,
    },
    error::Error,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use crate::{
    command::Command,
    config::*,
    error::*,
    monoframe::MonoFrame,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// command objectives
// v2df init
// v2df new
// v2df run
// v2df help / --help / -h
// v2df test --single_frame frame_num

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
    let mut args = env::args().skip(1);

    let command_str = args.next().ok_or(CliError::NoCommand)?;

    let command = Command::from_name(&command_str).ok_or(CliError::UnknownCommand(command_str))?;

    command.execute(args)
}

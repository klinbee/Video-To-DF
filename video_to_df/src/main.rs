mod command;
mod config;
mod error;
mod functions;
mod monovideo;

use std::{
    env,
    error::Error,
    fmt::Result as FormatResult,
    fs,
    path::PathBuf,
    time::Instant,
};

use crate::{
    command::Command,
    config::*,
    error::*,
};

/*
 * Don't forget!
 * cargo build --release
 * cargo install --path .
 */

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
    let mut args = env::args().skip(1);

    let command_str = args.next().ok_or(CliError::NoCommand)?;

    let command = Command::from_name(&command_str).ok_or(CliError::UnknownCommand(command_str))?;

    match command
    {
        Command::Init =>
        {
            let init_start = Instant::now();

            let path = args.next().map(PathBuf::from);

            let path = functions::get_path_or_curr_dir(path)?;

            println!("Creating v2df project in directory: {}", path.display());

            let config = Config::default();

            let config_path = path.join("v2df_config.json");
            fs::create_dir_all(&path)
                .map_err(|e| ImplError::CreateDirectory(format!("{:?}", e)))?;

            let config_content = serde_json::to_string_pretty(&config)
                .map_err(|e| ImplError::JsonPrettifier(format!("{:?}", e)))?;

            fs::write(config_path, config_content)
                .map_err(|e| ImplError::FileWrite(format!("{:?}", e)))?;

            let init_time = init_start.elapsed().as_millis();

            println!(
                "Successfully created v2df project in {}",
                functions::format_duration(init_time)
            );

            Ok(())
        },
        Command::Run =>
        {
            let run_start = Instant::now();

            let path = args.next().map(PathBuf::from);

            let path = functions::get_path_or_curr_dir(path)?;

            println!("Running v2df in directory: {}", path.display());

            let config = Config::from_path(&path.join("v2df_config.json"))?;

            let frames = functions::get_single_channel_frames(&config.video_file)?;

            functions::write_projects_from_config(frames, config)?;

            let run_time = run_start.elapsed().as_millis();

            println!("Successfully ran v2df project in {}", functions::format_duration(run_time));

            Ok(())
        },
        Command::Test =>
        {
            let test_start = Instant::now();

            let path = args.next().map(PathBuf::from);

            let path = functions::get_path_or_curr_dir(path)?;

            println!("Testing v2df in directory: {}", path.display());

            let config = Config::from_path(&path.join("v2df_config.json"))?;

            let frames = functions::get_single_channel_frames(&config.video_file)?;

            functions::test_projects_from_config(frames, config)?;

            let test_time = test_start.elapsed().as_millis();

            println!("Successfully ran v2df test in {}", functions::format_duration(test_time));

            Ok(())
        },
        Command::Help =>
        {
            println!(
                "Usage: v2df [COMMAND]

        COMMANDS:
            init [path]    Initialize a new project in the specified directory
                           If no path is provided, initializes in current directory

                           New projects consist of a default 'v2df_config.json' file

                           WARNING: overrides existing project configurations

            run [path]     Execute the project in the specified directory
                           If no path is provided, runs project in current directory
                           If no 'v2df_config.json' file is found in the current directory, exits
                           If no entry matching the 'video_file' field is found, exits

                           Running this project reads the 'v2df_config.json' and 'video_file'
                           The 'video_file' is:
                           - Processed into black and white frames (single channel, mono)
                           - Adds a border
                           - Applies a gradient
                           - Deflated
                           - 64bit Encoded
                           - Placed into a 'frame_<n>.json' density_function file

                           The density_function file uses the More Density Functions mod to
                           convert all the video's frames into data
                           that can be used as a heightmap for terrain in Minecraft

                           WARNING: overrides existing project files

            test [path]    Runs a single frame test for the project in the specified directory
                           If no path is provided, runs tests in current directory
                           If no 'v2df_config.json' file is found in the current directory, exits
                           If no entry matching the 'video_file' field is found, exits

                           The single frame test consists of:
                           - a 'frame_<n>.json'
                           - an 'all_frames.json' containing that frame's reference
                           - the frame image before processing
                           - the frame image after processing (gradient and border)

                           WARNING: overrides existing project files

            help           Show this help message

        ARGUMENTS:
            [path]         Optional path to target directory
                           Defaults to current directory if not specified

        EXAMPLES:
            v2df init                    # Initialize project in current directory
            v2df init ./my-project       # Initialize project in ./my-project
            v2df run                     # Run project in current directory
            v2df run ../other-project    # Run project in ../other-project
            v2df test ./src              # Run tests in ./src directory
            v2df help                    # Show this help message!"
            );
            Ok(())
        },
    }
}

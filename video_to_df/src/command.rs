use std::{
    env,
    fmt::{
        Display,
        Formatter,
    },
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use crate::{
    CliError,
    Config,
    FormatResult,
    ImplError,
    Result,
    output,
};

#[derive(Debug)]
pub enum Command
{
    Init,
    Run,
    Test,
    Help,
}

impl Command
{
    const HELP: &'static str = "help";
    const INIT: &'static str = "init";
    const RUN: &'static str = "run";
    const TEST: &'static str = "test";

    pub fn name(&self) -> &'static str
    {
        match self
        {
            Self::Init => Self::INIT,
            Self::Run => Self::RUN,
            Self::Test => Self::TEST,
            Self::Help => Self::HELP,
        }
    }

    pub fn from_name(name: &str) -> Option<Self>
    {
        match name
        {
            Self::INIT => Some(Self::Init),
            Self::RUN => Some(Self::Run),
            Self::TEST => Some(Self::Test),
            Self::HELP => Some(Self::Help),
            _ => None,
        }
    }

    pub fn execute(
        self,
        mut args: impl Iterator<Item = String>,
    ) -> Result<()>
    {
        match self
        {
            Self::Init => Self::execute_init(args.next().map(PathBuf::from)),
            Self::Run => Self::execute_run(args.next().map(PathBuf::from)),
            Self::Test => Self::execute_test(args.next().map(PathBuf::from)),
            Self::Help => Self::execute_help(),
        }
    }

    fn get_path_or_curr_dir(path: Option<PathBuf>) -> Result<PathBuf>
    {
        match path
        {
            None => env::current_dir().map_err(|_| CliError::AccessCurrentDirectory.into()),
            Some(path) => Ok(path),
        }
    }

    fn execute_init(path: Option<PathBuf>) -> Result<()>
    {
        let path = Self::get_path_or_curr_dir(path)?;
        println!("Creating project at: {}", path.display());

        let config = Config::default();

        let config_path = path.join("v2df_config.json");
        fs::create_dir_all(&path).map_err(|e| ImplError::CreateDirectory(e))?;
        let config_content =
            serde_json::to_string_pretty(&config).map_err(|e| ImplError::JsonPrettifier(e))?;
        fs::write(config_path, config_content).map_err(|e| ImplError::FileWrite(e))?;
        Ok(())
    }

    fn execute_run(path: Option<PathBuf>) -> Result<()>
    {
        let path = Self::get_path_or_curr_dir(path)?;
        println!("Attempting to run v2df in directory: {:?}", path);
        let config = Self::get_config(&path)?;
        let frames = output::get_single_channel_frames(&config.video_file)?;
        output::write_projects_from_config(frames, config)?;
        Ok(())
    }

    fn execute_test(path: Option<PathBuf>) -> Result<()>
    {
        let path = Self::get_path_or_curr_dir(path)?;
        println!("Attempting to test v2df in directory: {:?}", path);
        let config = Self::get_config(&path)?;
        let frames = output::get_single_channel_frames(&config.video_file)?;
        output::test_projects_from_config(frames, config)?;
        Ok(())
    }

    fn execute_help() -> Result<()>
    {
        println!("Usage: v2df [COMMAND]

    COMMANDS:
        init [path]    Initialize a new project in the specified directory
                       If no path is provided, initializes in current directory

                       New projects consist of a default `v2df_config.json` file
                       Warning: overrides existing project configurations

        run [path]     Execute the project defined by `v2df_config.json` in the specified directory
                       If no path is provided, runs project in current directory
                       If no `v2df_config.json` file is found in the current directory, exits
                       If no entry matching the `video_file` field is found, exits

                       Running this project reads the `v2df_config.json` file and the `video_file`
                       The `video_file` is processed into black and white frames, which then have a border added
                       and have a two pass signed distance field computed on them, resulting in a gradient

                       This data is then deflated and converted encoded into a 64bit string, and that string is
                       placed into a density_function .json file to be used by the More Density Functions mod to
                       convert all the video's frames into data that can be used as a heightmap for terrain in Minecraft

        test [path]    Runs a single frame test for the project in the specified directory
                       If no path is provided, runs tests in current directory
                       If no `v2df_config.json` file is found in the current directory, exits
                       If no entry matching the `video_file` field is found, exits

                       The single frame test consists of:
                       - a `frame.json`
                       - an `all_frames.json` containing that frame's reference
                       - the frame image before processing
                       - the frame image after processing (gradient and border)

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
        v2df help                    # Show this help message!");
        Ok(())
    }

    fn get_config(path: &Path) -> Result<Config>
    {
        let config_path = path.join("v2df_config.json");
        if !(config_path.exists() && config_path.is_file())
        {
            return Err(CliError::ConfigNotFound(path.to_owned()).into());
        }
        let config_str =
            fs::read_to_string(&config_path).map_err(|err| CliError::ConfigRead(err))?;
        let config: Config =
            serde_json::from_str(&config_str).map_err(|err| CliError::ConfigParse(err))?;
        Ok(config)
    }
}

impl Display for Command
{
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> FormatResult
    {
        write!(f, "{}", self.name())
    }
}

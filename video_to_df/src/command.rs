use crate::{
    CliError,
    Config,
    ImplError,
    Path,
    PathBuf,
    Result,
    env,
    fs,
    output::{
        get_single_channel_frames,
        test_projects_from_config,
        write_projects_from_config,
    },
};
#[derive(Debug)]
pub enum Command
{
    Init,
    Run,
    Test,
}

impl Command
{
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
        }
    }

    pub fn from_name(name: &str) -> Option<Self>
    {
        match name
        {
            Self::INIT => Some(Self::Init),
            Self::RUN => Some(Self::Run),
            Self::TEST => Some(Self::Test),
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
        std::fs::write(config_path, config_content).map_err(|e| ImplError::FileWrite(e))?;
        Ok(())
    }

    fn execute_run(path: Option<PathBuf>) -> Result<()>
    {
        let path = Self::get_path_or_curr_dir(path)?;
        println!("Attempting to run v2df in directory: {:?}", path);
        let config = Self::get_config(&path)?;
        let frames = get_single_channel_frames(&config.video_file)?;
        write_projects_from_config(frames, config)?;
        Ok(())
    }

    fn execute_test(path: Option<PathBuf>) -> Result<()>
    {
        let path = Self::get_path_or_curr_dir(path)?;
        println!("Attempting to test v2df in directory: {:?}", path);
        let config = Self::get_config(&path)?;
        let frames = get_single_channel_frames(&config.video_file)?;
        test_projects_from_config(frames, config)?;
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

impl std::fmt::Display for Command
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result
    {
        write!(f, "{}", self.name())
    }
}

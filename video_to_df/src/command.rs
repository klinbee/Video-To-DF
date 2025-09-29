use std::fmt::{
    Display,
    Formatter,
};

use crate::FormatResult;

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
        for cmd in [Self::Init, Self::Run, Self::Test, Self::Help]
        {
            if name == cmd.name() || name == cmd.alias_short() || name == cmd.alias_long()
            {
                return Some(cmd);
            }
        }
        None
    }

    fn alias_short(&self) -> String
    {
        format!("-{}", self.name().chars().next().unwrap())
    }

    fn alias_long(&self) -> String
    {
        format!("--{}", self.name())
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

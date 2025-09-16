use std::{
    error::Error,
    fmt::{
        Display,
        Formatter,
    },
    path::PathBuf,
};

use crate::{
    FFmpegError,
    FormatResult,
    IoError,
    SerdeJsonError,
};

#[derive(Debug)]
pub enum CliError
{
    NoCommand,
    UnknownCommand(String),
    ConfigNotFound(PathBuf),
    ConfigRead(IoError),
    ConfigParse(SerdeJsonError),
    InvalidFrameRange((usize, usize), usize),
    AccessCurrentDirectory,
    InvalidTestFrame(usize, usize),
}

impl Error for CliError {}

impl Display for CliError
{
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> FormatResult
    {
        write!(f, "v2df: ")?;
        match self
        {
            Self::NoCommand => write!(f, "Type --help for usage"),
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: '{}'", cmd),
            Self::ConfigNotFound(path) =>
            {
                write!(f, "Failed to find 'v2df_config.json' in directory: {}", path.display())
            },
            Self::ConfigParse(serde_err) =>
            {
                write!(f, "Failed to parse 'v2df_config.json': {}", serde_err)
            },
            Self::AccessCurrentDirectory => write!(f, "Could not access current directory"),
            Self::ConfigRead(io_err) =>
            {
                write!(f, "Failed to read 'v2df_config.json': {}", io_err)
            },
            Self::InvalidFrameRange(frame_range, frame_count) =>
            {
                write!(
                    f,
                    "Frame range [{}, {}] is out of range of frame count: {}",
                    frame_range.0, frame_range.1, frame_count
                )
            },
            Self::InvalidTestFrame(test_frame, frame_count) =>
            {
                write!(
                    f,
                    "Test frame {} is out of range of frame count: {}",
                    test_frame, frame_count
                )
            },
        }?;
        writeln!(f)
    }
}

#[derive(Debug)]
pub enum ImplError
{
    AccessProjectConfig,
    ImageCreation,
    ImageSaving,
    JsonPrettifier(SerdeJsonError),
    FileCompression(IoError),
    FileWrite(IoError),
    FetchVideoStream,
    CreateDirectory(IoError),
    FFmpeg(FFmpegError),
}

impl Error for ImplError {}

impl Display for ImplError
{
    fn fmt(
        &self,
        f: &mut Formatter,
    ) -> FormatResult
    {
        write!(f, "v2df: ")?;
        match self
        {
            Self::AccessProjectConfig =>
            {
                write!(f, "Somehow failed to acess the project config from config")
            },
            Self::ImageCreation => write!(f, "Somehow failed to create image"),
            Self::ImageSaving => write!(f, "Somehow failed to save image"),
            Self::JsonPrettifier(e) =>
            {
                write!(f, "Somehow failed to prettify the output JSON: {}", e)
            },
            Self::FileCompression(e) =>
            {
                write!(f, "Somnehow failed during zlib compression: {}", e)
            },
            Self::FetchVideoStream => write!(f, "Somehow failed to fetch video stream"),
            Self::FFmpeg(e) => write!(f, "Somehow failed during video processing: {}", e),
            Self::FileWrite(e) => write!(f, "Somehow failed to write file during output: {}", e),
            Self::CreateDirectory(e) =>
            {
                write!(f, "Somehow failed to create directory during output: {}", e)
            },
        }?;
        writeln!(f)
    }
}

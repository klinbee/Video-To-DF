use std::{
    env::{
        self,
    },
    error::Error,
    fs,
    io::{
        self,
        Write,
    },
    num::NonZeroU32,
    path::{
        Path,
        PathBuf,
    },
    usize,
};

use base64::{
    Engine as _,
    engine::general_purpose,
};
use ffmpeg_next as ffmpeg;
use flate2::{
    Compression,
    write::ZlibEncoder,
};
use image::{
    ImageBuffer,
    Luma,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// command objectives
// v2df init
// v2df new
// v2df run
// v2df help / --help / -h
// v2df test --single_frame frame_num
impl Error for CliError {}
impl Error for ImplError {}

#[derive(Debug)]
enum CliError
{
    NoCommand,
    UnknownCommand(String),
    MissingArg(Command, String),
    InvalidRunDirectory(PathBuf),
    ConfigNotFound(PathBuf),
    ConfigRead(io::Error),
    ConfigParse(serde_json::Error),
    InvalidFrameRange((NonZeroU32, NonZeroU32), usize),
    AccessCurrentDirectory,
}

impl std::fmt::Display for CliError
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result
    {
        match self
        {
            Self::NoCommand => write!(f, "Type --help for usage"),
            Self::UnknownCommand(cmd) => write!(f, "Unknown command: '{}'", cmd),
            Self::MissingArg(command, arg_name) =>
            {
                write!(f, "'{}' command requires a {}", command, arg_name)
            },
            Self::ConfigNotFound(path) =>
            {
                write!(f, "Failed to find 'v2df_config.json' in directory: {}", path.display())
            },
            Self::InvalidRunDirectory(path) =>
            {
                write!(f, "Something is wrong with the current directory: {}", path.display())
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
            Self::InvalidFrameRange(frame_range, max_frames) =>
            {
                write!(
                    f,
                    "Frame range [{}, {}] is out of range of frame count: {}",
                    frame_range.0, frame_range.1, max_frames
                )
            },
        }
    }
}

#[derive(Debug)]
enum ImplError
{
    AccessProjectConfig,
    ImageCreation,
    ImageSaving,
    JsonPrettifier(serde_json::Error),
    FileCompression(io::Error),
    FileWrite(io::Error),
    FetchVideoStream,
    CreateDirectory(io::Error),
    FFmpeg(ffmpeg::Error),
}

impl std::fmt::Display for ImplError
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result
    {
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
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config
{
    video_file: PathBuf,
    output_root_dir: PathBuf,
    projects: Vec<ProjectConfig>,
}

#[derive(Serialize, Deserialize)]
struct ProjectConfig
{
    border_width: u16,
    border_color: u8,
    invert_colors: bool,
    frame_start: NonZeroU32,
    frame_end: NonZeroU32,
    frame_dfs_dir: PathBuf,
    grid_df_dir: PathBuf,
    test_frame: NonZeroU32,
}

#[derive(Debug)]
enum Command
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

    fn name(&self) -> &'static str
    {
        match self
        {
            Self::Init => Self::INIT,
            Self::Run => Self::RUN,
            Self::Test => Self::TEST,
        }
    }

    fn from_name(name: &str) -> Option<Self>
    {
        match name
        {
            Self::INIT => Some(Self::Init),
            Self::RUN => Some(Self::Run),
            Self::TEST => Some(Self::Test),
            _ => None,
        }
    }

    fn execute(
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
        println!("Creating project at: {:?}", path);
        todo!()
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

fn main() -> Result<()>
{
    let mut args = env::args().skip(1);

    let command_str = args.next().ok_or(CliError::NoCommand)?;

    let command = Command::from_name(&command_str).ok_or(CliError::UnknownCommand(command_str))?;

    command.execute(args)
}

fn write_projects_from_config(
    frames: Vec<MonoFrame>,
    config: Config,
) -> Result<()>
{
    let num_projects = config.projects.len();
    fs::create_dir_all(&config.output_root_dir).map_err(|e| ImplError::CreateDirectory(e))?;
    for n in 0..num_projects
    {
        write_project_n_from_config(&frames, n, &config)?;
    }
    Ok(())
}

fn write_project_n_from_config(
    frames: &Vec<MonoFrame>,
    n: usize,
    config: &Config,
) -> Result<()>
{
    let project_config = config.projects.get(n).ok_or(ImplError::AccessProjectConfig)?;
    let frame_dim = (frames[0].width as usize, frames[0].height as usize);
    let root_dir = &config.output_root_dir;
    let frame_dir = root_dir.join(&project_config.frame_dfs_dir);
    let grid_dir = root_dir.join(&project_config.grid_df_dir);
    let frame_range = (project_config.frame_start, project_config.frame_end);
    write_json_frames(
        frames,
        frame_dim,
        frame_range,
        project_config.border_width,
        project_config.border_color,
        &frame_dir,
    )?;
    write_json_grid(frame_range, frame_dim, &grid_dir)?;
    Ok(())
}

fn test_projects_from_config(
    frames: Vec<MonoFrame>,
    config: Config,
) -> Result<()>
{
    let num_projects = config.projects.len();
    fs::create_dir_all(&config.output_root_dir).map_err(|e| ImplError::CreateDirectory(e))?;
    for n in 0..num_projects
    {
        test_project_n_from_config(&frames, n, &config)?;
    }
    Ok(())
}

fn test_project_n_from_config(
    frames: &Vec<MonoFrame>,
    n: usize,
    config: &Config,
) -> Result<()>
{
    let project_config = config.projects.get(n).ok_or(ImplError::AccessProjectConfig)?;
    let frame_dim = (frames[0].width as usize, frames[0].height as usize);
    let root_dir = &config.output_root_dir;
    let frame_dir = root_dir.join(&project_config.frame_dfs_dir);
    let grid_dir = root_dir.join(&project_config.grid_df_dir);
    let frame_range = (project_config.test_frame, project_config.test_frame.saturating_add(1));
    write_json_frames(
        frames,
        frame_dim,
        frame_range,
        project_config.border_width,
        project_config.border_color,
        &frame_dir,
    )?;
    write_json_grid(frame_range, frame_dim, &grid_dir)?;
    Ok(())
}

fn write_json_frames(
    frames: &Vec<MonoFrame>,
    frame_dim: (usize, usize),
    frame_range: (NonZeroU32, NonZeroU32),
    border_width: u16,
    border_color: u8,
    output_dir: &Path,
) -> Result<()>
{
    let index_start = (frame_range.0.get() - 1) as usize;
    let index_end = (frame_range.0.get() - 1) as usize;
    fs::create_dir_all(&output_dir).map_err(|e| ImplError::CreateDirectory(e))?;
    if index_start.min(index_end) > frames.len()
    {
        return Err(CliError::InvalidFrameRange(frame_range, frames.len()).into());
    }
    for (index, frame) in (index_start..index_end).zip(frames.iter().skip(index_start))
    {
        let grad_frame = binary_sdf(&frame.add_border(border_width, border_color));
        let deflated_grad_frame = compress_zlib(&grad_frame.data)?;
        let encoded_deflated_grad_frame_data =
            general_purpose::STANDARD.encode(&deflated_grad_frame);
        let frame_json = json!(
            {
                "type": "moredfs:single_channel_image_tessellation",
                "x_size": frame_dim.0,
                "z_size": frame_dim.1,
                "deflated_frame_data": encoded_deflated_grad_frame_data
            }
        );
        let frame_json_string =
            serde_json::to_string_pretty(&frame_json).map_err(|e| ImplError::JsonPrettifier(e))?;
        fs::write(output_dir.join(&format!("frame_{}.json", index)), &frame_json_string)
            .map_err(|e| ImplError::FileWrite(e))?;
    }
    Ok(())
}

fn write_json_grid(
    frame_range: (NonZeroU32, NonZeroU32),
    frame_dim: (usize, usize),
    output_dir: &Path,
) -> Result<()>
{
    fs::create_dir_all(&output_dir).map_err(|e| ImplError::CreateDirectory(e))?;
    let frame_json = json!(
        {
            "type": "moredfs:gapped_grid_square_spiral",
            "spacing": 2,
            "x_size":  frame_dim.0,
            "z_size": frame_dim.1,
            "out_of_bounds_argument": -1,
            "grid_cell_args": (frame_range.0.get()..=frame_range.1.get())
                .map(|i| format!("term{}", i))
                .collect::<Vec<_>>()
        }
    );
    let frame_json_string =
        serde_json::to_string_pretty(&frame_json).map_err(|e| ImplError::JsonPrettifier(e))?;
    fs::write(output_dir.join("all_frames.json"), &frame_json_string)
        .map_err(|e| ImplError::FileWrite(e))?;
    Ok(())
}

fn compress_zlib(bytes: &[u8]) -> Result<Vec<u8>>
{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).map_err(|e| ImplError::FileCompression(e))?;
    let compressed_bytes = encoder.finish().map_err(|e| ImplError::FileCompression(e))?;
    Ok(compressed_bytes)
}

pub struct MonoFrame
{
    data: Vec<u8>,
    width: u16,
    height: u16,
}

impl MonoFrame
{
    fn new(
        data: Vec<u8>,
        width: u16,
        height: u16,
    ) -> MonoFrame
    {
        MonoFrame {
            data,
            width,
            height,
        }
    }

    fn solid_color(
        width: u16,
        height: u16,
        color: u8,
    ) -> MonoFrame
    {
        MonoFrame {
            data: vec![color; width as usize * height as usize],
            width,
            height,
        }
    }

    fn add_border(
        &self,
        border_width: u16,
        border_color: u8,
    ) -> MonoFrame
    {
        let new_width = self.width + 2 * border_width;
        let new_height = self.height + 2 * border_width;

        let mut with_border = MonoFrame::solid_color(new_width, new_height, border_color);

        for y in 0..self.height
        {
            let src_start = y as usize * self.width as usize;
            let src_end = src_start + self.width as usize;
            let dst_start =
                ((y as usize + border_width as usize) * new_width as usize) + border_width as usize;
            let dst_end = dst_start + self.width as usize;

            with_border.data[dst_start..dst_end].copy_from_slice(&self.data[src_start..src_end]);
        }
        with_border
    }

    fn save_as(
        &self,
        filename: &str,
    ) -> Result<()>
    {
        // Create image buffer from monochromatic data
        let mut img_data = Vec::with_capacity(self.width as usize * self.height as usize);

        // Copy data row by row to handle stride
        for y in 0..self.height
        {
            let row_start = y as usize * self.width as usize;
            let row_end = row_start as usize + self.width as usize;
            img_data.extend_from_slice(&self.data[row_start..row_end]);
        }

        let img: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_raw(self.width as u32, self.height as u32, img_data)
                .ok_or(ImplError::ImageCreation)?;

        img.save(filename).map_err(|_| ImplError::ImageSaving)?;
        println!("Saved PNG to {}", filename);
        Ok(())
    }
}

fn get_single_channel_frames<P>(video_path: P) -> Result<Vec<MonoFrame>>
where
    P: AsRef<Path>,
{
    ffmpeg::init().map_err(|e| ImplError::FFmpeg(e))?;

    let mut input = ffmpeg::format::input(video_path.as_ref()).map_err(|e| ImplError::FFmpeg(e))?;

    let video_stream =
        input.streams().best(ffmpeg::media::Type::Video).ok_or(ImplError::FetchVideoStream)?;

    let video_stream_index = video_stream.index();

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())
        .map_err(|e| ImplError::FFmpeg(e))?
        .decoder()
        .video()
        .map_err(|e| ImplError::FFmpeg(e))?;

    // Set up context to convert to monochromatic
    let mut monochromatic_ctx = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::GRAY8, // Single channel monochromatic
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )
    .map_err(|e| ImplError::FFmpeg(e))?;

    let mut frames: Vec<MonoFrame> = vec![];

    for (stream, packet) in input.packets()
    {
        if stream.index() == video_stream_index
        {
            decoder.send_packet(&packet).map_err(|e| ImplError::FFmpeg(e))?;

            let mut decoded = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok()
            {
                let mut mono_video = ffmpeg::util::frame::video::Video::empty();

                monochromatic_ctx
                    .run(&decoded, &mut mono_video)
                    .map_err(|e| ImplError::FFmpeg(e))?;

                frames.push(MonoFrame::new(
                    mono_video.data(0).to_vec(), // Single channel data
                    mono_video.width() as u16,
                    mono_video.height() as u16,
                ));
            }
        }
    }
    // Flush decoder (could be storing extra frames)
    decoder.send_eof().map_err(|e| ImplError::FFmpeg(e))?;
    let mut decoded = ffmpeg::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok()
    {
        let mut mono_video = ffmpeg::util::frame::video::Video::empty();
        monochromatic_ctx.run(&decoded, &mut mono_video).map_err(|e| ImplError::FFmpeg(e))?;

        frames.push(MonoFrame::new(
            mono_video.data(0).to_vec(),
            mono_video.width() as u16,
            mono_video.height() as u16,
        ));
    }
    Ok(frames)
}

fn binary_sdf(frame: &MonoFrame) -> MonoFrame
{
    // First, compute the normal sdf
    let sdf_raw = chebyshev_sdf_two_pass(
        &frame.data,
        frame.width as usize,
        frame.height as usize,
        127, // Splits 0-127 & 128-255
    );

    // Then, find the `max_value` in it
    let max_value: usize = *sdf_raw.iter().max().expect("SDF Raw should never have size 0");

    // If the `max_value` is 0, then use all white (as the SDF value is inverted)
    if max_value == 0
    {
        return MonoFrame::new(vec![255; sdf_raw.len()], frame.width, frame.height);
    }

    // Then, convert the `sdf_raw` from `usize` to `u8` by normalizing to `max_value` and clamping
    let sdf_bytes = sdf_raw
        .iter()
        .map(|&val| {
            let norm = 1.0 - (val as f32 / max_value as f32);
            (norm * 255.0).round().clamp(0.0, 255.0) as u8
        })
        .collect();

    // Return it as a MonoFrame
    MonoFrame::new(sdf_bytes, frame.width, frame.height)
}

fn chebyshev_sdf_two_pass(
    image: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Vec<usize>
{
    let mut distance_field: Vec<usize> = vec![usize::MAX; width * height];

    // Sets the distance field value at that position to 0 where the pixel value is above threshold
    distance_field.iter_mut().zip(image.iter()).for_each(|(dist_val, pixel_val)| {
        if pixel_val > &threshold
        {
            *dist_val = 0;
        }
    });

    chebyshev_sdf_forward_pass(&mut distance_field, width, height);

    // Better access pattern to reverse all at once and walk forward
    distance_field.reverse();
    chebyshev_sdf_forward_pass(&mut distance_field, width, height);

    // Change to normal order
    distance_field.reverse();

    distance_field
}

fn chebyshev_sdf_forward_pass(
    distance_field: &mut Vec<usize>,
    width: usize,
    height: usize,
)
{
    // Forward pass (row-wise, column-wise, diagonal-wise)
    let mut idx = 0;
    for y in 0..height
    {
        for x in 0..width
        {
            let mut curr_dist = distance_field[idx];

            // Top-left Diagonal (if within bounds)

            if x > 0 && y > 0
            {
                let n_dist = distance_field[idx - width - 1];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist
                {
                    curr_dist = new_dist;
                }
            }

            // Top (if within bounds)
            if y > 0
            {
                let n_dist = distance_field[idx - width];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist
                {
                    curr_dist = new_dist;
                }
            }

            // Left (if within bounds)
            if x > 0
            {
                let n_dist = distance_field[idx - 1];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist
                {
                    curr_dist = new_dist;
                }
            }

            distance_field[idx] = curr_dist;
            idx += 1;
        }
    }
}

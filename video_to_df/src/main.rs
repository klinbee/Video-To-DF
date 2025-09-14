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
impl Error for IoError {}
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
        }
    }
}

#[derive(Debug)]
enum IoError
{
    FileCompression,
}

impl std::fmt::Display for IoError
{
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result
    {
        match self
        {
            Self::FileCompression => write!(f, "Failed during zlib compression step..."),
        }
    }
}

#[derive(Debug)]
enum ImplError
{
    AccessProjectConfig,
    ImageCreation,
    ImageSaving,
    JsonPrettifier,
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
            Self::JsonPrettifier => write!(f, "Somehow failed to prettify the output JSON"),
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
    frame_start: u32,
    frame_dfs_dir: PathBuf,
    grid_df_dir: PathBuf,
}

#[derive(Debug)]
enum Command
{
    Init,
    New,
    Run,
}

impl Command
{
    const INIT: &'static str = "init";
    const NEW: &'static str = "new";
    const RUN: &'static str = "run";

    fn name(&self) -> &'static str
    {
        match self
        {
            Command::Init => Self::INIT,
            Command::New => Self::NEW,
            Command::Run => Self::RUN,
        }
    }

    fn from_name(name: &str) -> Option<Self>
    {
        match name
        {
            Self::INIT => Some(Command::Init),
            Self::NEW => Some(Command::New),
            Self::RUN => Some(Command::Run),
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
            Command::Init => Self::execute_new(&env::current_dir()?),
            Command::New =>
            {
                let project_dir = args
                    .next()
                    .ok_or(CliError::MissingArg(Command::New, "project directory".to_string()))?;
                Self::execute_new(&PathBuf::from(project_dir))
            },
            Command::Run => Self::execute_run(args.next().map(PathBuf::from)),
        }
    }

    fn execute_new(path: &Path) -> Result<()>
    {
        println!("Creating project at: {:?}", path);
        todo!()
    }

    fn execute_run(path: Option<PathBuf>) -> Result<()>
    {
        let path =
            path.or_else(|| env::current_dir().ok()).ok_or(CliError::AccessCurrentDirectory)?;
        println!("Attempting to run v2df in directory: {:?}", path);
        let config_file = Self::get_config_file(&path)?;
        let config_str =
            fs::read_to_string(&config_file).map_err(|err| CliError::ConfigRead(err))?;
        let config: Config =
            serde_json::from_str(&config_str).map_err(|err| CliError::ConfigParse(err))?;
        let frames = get_single_channel_frames(&config.video_file)?;
        write_projects_from_config(frames, config)?;
        Ok(())
    }

    fn get_config_file(path: &Path) -> Result<PathBuf>
    {
        let config_file = path.join("v2df_config.json");
        if config_file.exists() && config_file.is_file()
        {
            return Ok(config_file);
        }
        Err(CliError::ConfigNotFound(path.to_owned()).into())
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

// let frames = get_single_channel_frames("Bad_Apple!!.mp4")?;
// single_frame_test(frames, 1337)

fn single_frame_test(
    frames: Vec<MonoFrame>,
    target_frame: usize,
) -> Result<()>
{
    let my_frame = frames.get(target_frame).expect("Frame not found!");
    my_frame.save_as(&format!("frame_{}.png", target_frame))?;

    let grad_frame = binary_sdf(&my_frame.add_border(30, 255));

    grad_frame.save_as(&format!("frame_grad_{}.png", target_frame))?;

    let deflated_grad_frame = compress_zlib(&grad_frame.data)?;
    let encoded_deflated_grad_frame_data = general_purpose::STANDARD.encode(&deflated_grad_frame);
    let frame_json = json!(
        {
            "type": "moredfs:single_channel_image_tessellation",
            "x_size": grad_frame.width,
            "z_size": grad_frame.height,
            "deflated_frame_data": encoded_deflated_grad_frame_data
        }
    );

    let frame_json_string =
        serde_json::to_string_pretty(&frame_json).map_err(|_| ImplError::JsonPrettifier)?;

    fs::write("frame.json", &frame_json_string)?;
    Ok(())
}

fn write_projects_from_config(
    frames: Vec<MonoFrame>,
    config: Config,
) -> Result<()>
{
    let num_projects = config.projects.len();
    fs::create_dir_all(&config.output_root_dir)?;
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
    let x_size: usize = frames[0].width as usize;
    let z_size: usize = frames[0].height as usize;
    let root_dir = &config.output_root_dir;
    write_json_frames_from_config(&frames, x_size, z_size, root_dir, &project_config)?;
    write_json_grid_from_config(frames.len(), x_size, z_size, root_dir, &project_config)?;
    Ok(())
}

fn write_json_frames_from_config(
    frames: &Vec<MonoFrame>,
    x_size: usize,
    z_size: usize,
    root_dir: &Path,
    project_config: &ProjectConfig,
) -> Result<()>
{
    let frame_start = project_config.frame_start;
    for (index, frame) in (frame_start..).zip(frames.iter().skip(frame_start as usize - 1))
    {
        let grad_frame =
            binary_sdf(&frame.add_border(project_config.border_width, project_config.border_color));
        let deflated_grad_frame = compress_zlib(&grad_frame.data)?;
        let encoded_deflated_grad_frame_data =
            general_purpose::STANDARD.encode(&deflated_grad_frame);
        let frame_json = json!(
            {
                "type": "moredfs:single_channel_image_tessellation",
                "x_size": x_size,
                "z_size": z_size,
                "deflated_frame_data": encoded_deflated_grad_frame_data
            }
        );
        let frame_json_string = serde_json::to_string_pretty(&frame_json)?;
        fs::write(
            root_dir.join(&project_config.frame_dfs_dir).join(&format!("frame_{}.json", index)),
            &frame_json_string,
        )?;
    }
    Ok(())
}

fn write_json_grid_from_config(
    frame_count: usize,
    x_size: usize,
    z_size: usize,
    root_dir: &Path,
    project_config: &ProjectConfig,
) -> Result<()>
{
    let frame_json = json!(
        {
            "type": "moredfs:gapped_grid_square_spiral",
            "spacing": 2,
            "x_size": x_size,
            "z_size": z_size,
            "out_of_bounds_argument": -1,
            "grid_cell_args": (1..=frame_count)
                .map(|i| format!("term{}", i))
                .collect::<Vec<_>>()
        }
    );
    let frame_json_string = serde_json::to_string_pretty(&frame_json)?;
    fs::write(
        root_dir.join(&project_config.grid_df_dir).join("all_frames.json"),
        &frame_json_string,
    )?;
    Ok(())
}

fn compress_zlib(bytes: &[u8]) -> Result<Vec<u8>>
{
    let result: Result<Vec<u8>> = {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(bytes)?;
        Ok(encoder.finish()?)
    };

    result.map_err(|_| IoError::FileCompression.into())
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
    ffmpeg::init()?;

    let mut input = ffmpeg::format::input(video_path.as_ref())?;

    let video_stream =
        input.streams().best(ffmpeg::media::Type::Video).expect("No video stream found");

    let video_stream_index = video_stream.index();

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?
        .decoder()
        .video()?;

    // Set up context to convert to monochromatic
    let mut monochromatic_ctx = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::GRAY8, // Single channel monochromatic
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )?;

    let mut frames: Vec<MonoFrame> = vec![];

    for (stream, packet) in input.packets()
    {
        if stream.index() == video_stream_index
        {
            decoder.send_packet(&packet)?;

            let mut decoded = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok()
            {
                let mut mono_video = ffmpeg::util::frame::video::Video::empty();

                monochromatic_ctx.run(&decoded, &mut mono_video)?;

                frames.push(MonoFrame::new(
                    mono_video.data(0).to_vec(), // Single channel data
                    mono_video.width() as u16,
                    mono_video.height() as u16,
                ));
            }
        }
    }
    // Flush decoder (could be storing extra frames)
    decoder.send_eof()?;
    let mut decoded = ffmpeg::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok()
    {
        let mut mono_video = ffmpeg::util::frame::video::Video::empty();
        monochromatic_ctx.run(&decoded, &mut mono_video)?;

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
    let max_value: usize = *sdf_raw.iter().max().unwrap_or(&0);

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

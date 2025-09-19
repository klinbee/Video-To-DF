use std::{
    fs,
    io::Write,
    path::Path,
};

use base64::{
    Engine as _,
    engine::general_purpose,
};
use flate2::{
    Compression,
    write::ZlibEncoder,
};
use serde_json::json;

use crate::{
    CliError,
    Config,
    ImplError,
    MonoFrame,
    Result,
    ffmpeg,
    sdf,
};

pub fn write_projects_from_config(
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

    let border_width = project_config.border_width as usize;

    let frame_dim =
        (frames[0].width as usize + border_width * 2, frames[0].height as usize + border_width * 2);

    let root_dir = &config.output_root_dir;

    let frame_dir = root_dir.join(&project_config.frame_dfs_dir);

    let grid_dir = root_dir.join(&project_config.grid_df_dir);

    let tp_dir = root_dir.join(&project_config.tp_dir);

    let index_start = match project_config.frame_start
    {
        None => 0,
        Some(frame_start) => (frame_start.get() - 1) as usize,
    };
    let index_end = match project_config.frame_end
    {
        None => frames.len(),
        Some(frame_start) => (frame_start.get() - 1) as usize,
    };

    if index_start.min(index_end) > frames.len()
    {
        return Err(
            CliError::InvalidFrameRange((index_start + 1, index_end + 1), frames.len()).into()
        );
    }

    let index_range = (index_start, index_end);

    let frame_namespace =
        create_df_namespace(&project_config.namespace, &project_config.frame_dfs_dir);

    if project_config.make_frames
    {
        write_json_frames(
            frames,
            frame_dim,
            index_range,
            border_width as u16,
            project_config.border_color,
            &frame_dir,
        )?;
    }

    if project_config.make_grid
    {
        write_json_grid(index_range, frame_dim, &frame_namespace, &grid_dir)?;
    }

    if project_config.make_tp
    {
        write_tp_functions(index_range, frame_dim, project_config.tp_height, &tp_dir)?;
    }
    Ok(())
}

fn create_df_namespace(
    namespace: &str,
    relative_path: &Path,
) -> String
{
    let relative_part = relative_path.strip_prefix("./").unwrap().to_string_lossy();

    format!("{}:{}/", namespace, relative_part)
}

pub fn test_projects_from_config(
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

    let border_width = project_config.border_width as usize;

    let frame_dim =
        (frames[0].width as usize + border_width * 2, frames[0].height as usize + border_width * 2);

    let root_dir = &config.output_root_dir;
    let frame_dir = root_dir.join(&project_config.frame_dfs_dir);
    let grid_dir = root_dir.join(&project_config.grid_df_dir);
    let tp_dir = root_dir.join(&project_config.tp_dir);

    let test_frame_index = match project_config.test_frame
    {
        None => 0,
        Some(test_frame) => (test_frame.get() - 1) as usize,
    };

    let target_frame = frames
        .get(test_frame_index)
        .ok_or(CliError::InvalidTestFrame(test_frame_index + 1, frames.len()))?;

    let index_range = (test_frame_index, test_frame_index + 1);

    target_frame.save_as(&root_dir.join(&format!("test_frame_{}.png", test_frame_index + 1)))?;

    sdf::binary_sdf(
        &target_frame.add_border(project_config.border_width, project_config.border_color),
    )
    .save_as(&root_dir.join(&format!("gradated_test_frame_{}.png", test_frame_index + 1)))?;

    let frame_namespace =
        create_df_namespace(&project_config.namespace, &project_config.frame_dfs_dir);

    if project_config.make_frames
    {
        write_json_frames(
            frames,
            frame_dim,
            index_range,
            border_width as u16,
            project_config.border_color,
            &frame_dir,
        )?;
    }

    if project_config.make_grid
    {
        write_json_grid(index_range, frame_dim, &frame_namespace, &grid_dir)?;
    }

    if project_config.make_tp
    {
        write_tp_functions(index_range, frame_dim, project_config.tp_height, &tp_dir)?;
    }

    Ok(())
}

fn write_json_frames(
    frames: &Vec<MonoFrame>,
    frame_dim: (usize, usize),
    index_range: (usize, usize),
    border_width: u16,
    border_color: u8,
    output_dir: &Path,
) -> Result<()>
{
    fs::create_dir_all(&output_dir).map_err(|e| ImplError::CreateDirectory(e))?;

    for (index, frame) in (index_range.0..index_range.1).zip(frames.iter().skip(index_range.0))
    {
        let grad_frame = sdf::binary_sdf(&frame.add_border(border_width, border_color));
        let deflated_grad_frame = compress_zlib(&grad_frame.data)?;
        let encoded_deflated_grad_frame_data =
            general_purpose::STANDARD.encode(&deflated_grad_frame);
        let frame_json = json!(
            {
                "type": "minecraft:flat_cache",
                "argument": {
                  "type": "minecraft:cache_2d",
                  "argument": {
                    "type": "moredfs:single_channel_image_tessellation",
                    "x_size": frame_dim.0,
                    "z_size": frame_dim.1,
                    "deflated_frame_data": encoded_deflated_grad_frame_data
                  }
                }
            }
        );
        let frame_json_string =
            serde_json::to_string_pretty(&frame_json).map_err(|e| ImplError::JsonPrettifier(e))?;
        fs::write(output_dir.join(&format!("{}.json", index + 1)), &frame_json_string)
            .map_err(|e| ImplError::FileWrite(e))?;
    }
    Ok(())
}

fn write_json_grid(
    index_range: (usize, usize),
    frame_dim: (usize, usize),
    namespace: &str,
    output_dir: &Path,
) -> Result<()>
{
    fs::create_dir_all(&output_dir).map_err(|e| ImplError::CreateDirectory(e))?;
    let frame_json = json!(
        {
            "type": "moredfs:gapped_grid_square_spiral",
            "spacing": 1,
            "x_size":  frame_dim.0,
            "z_size": frame_dim.1,
            "out_of_bounds_argument": 256,
            "grid_cell_args": ((index_range.0+1)..index_range.1)
                .map(|i| format!("{}{}", namespace,  i))
                .collect::<Vec<_>>()
        }
    );
    let frame_json_string =
        serde_json::to_string_pretty(&frame_json).map_err(|e| ImplError::JsonPrettifier(e))?;
    fs::write(output_dir.join("all_frames.json"), &frame_json_string)
        .map_err(|e| ImplError::FileWrite(e))?;
    Ok(())
}

fn write_tp_functions(
    index_range: (usize, usize),
    frame_dim: (usize, usize),
    tp_height: i16,
    output_dir: &Path,
) -> Result<()>
{
    fs::create_dir_all(&output_dir).map_err(|e| ImplError::CreateDirectory(e))?;

    for i in (index_range.0)..=index_range.1
    {
        let (curr_x, curr_z) = index_to_spiral_coords(i);
        let (curr_x, curr_z) = (
            curr_x * 2 * frame_dim.0 as isize + frame_dim.0 as isize / 2,
            curr_z * 2 * frame_dim.1 as isize + frame_dim.1 as isize / 2,
        );
        let tp_string = format!("tp @a {} {} {} 180 90", curr_x, tp_height, curr_z);
        fs::write(output_dir.join(format!("{}.mcfunction", i + 1)), &tp_string)
            .map_err(|e| ImplError::FileWrite(e))?;
    }
    Ok(())
}

fn index_to_spiral_coords(n: usize) -> (isize, isize)
{
    if n == 0
    {
        return (0, 0);
    }

    // Find which ring/layer we're in
    let layer = ((((n as f64).sqrt() - 1.0) / 2.0).floor() as isize) + 1;

    // Find the starting index of this layer
    let layer_start = (2 * layer - 1).pow(2);

    // Position within the layer
    let pos_in_layer = n as isize - layer_start;

    // Side length of current layer
    let side_length = 2 * layer;

    // Determine which side of the square we're on and calculate coordinates
    if pos_in_layer < side_length
    {
        // Right side, moving up
        (layer, -layer + 1 + pos_in_layer)
    }
    else if pos_in_layer < 2 * side_length
    {
        // Top side, moving left
        (layer - 1 - (pos_in_layer - side_length), layer)
    }
    else if pos_in_layer < 3 * side_length
    {
        // Left side, moving down
        (-layer, layer - 1 - (pos_in_layer - 2 * side_length))
    }
    else
    {
        // Bottom side, moving right
        (-layer + 1 + (pos_in_layer - 3 * side_length), -layer)
    }
}

fn compress_zlib(bytes: &[u8]) -> Result<Vec<u8>>
{
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).map_err(|e| ImplError::FileCompression(e))?;
    let compressed_bytes = encoder.finish().map_err(|e| ImplError::FileCompression(e))?;
    Ok(compressed_bytes)
}

pub fn get_single_channel_frames<P>(video_path: P) -> Result<Vec<MonoFrame>>
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

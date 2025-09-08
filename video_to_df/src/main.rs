use base64::{Engine as _, engine::general_purpose};
use core::f32;
use ffmpeg_next as ffmpeg;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde_json::json;
use std::io::Write;
use std::path::Path;
use std::{fs, usize};

// Generic Result
type GResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> GResult<()> {
    let frames = get_single_channel_frames("Bad_Apple!!.mp4")?;
    single_frame_test(frames, 1337)
}

fn single_frame_test(frames: Vec<MonoFrame>, target_frame: usize) -> GResult<()> {
    let my_frame = frames.get(target_frame).ok_or("Frame not found!")?;
    my_frame.save_as(&format!("frame_{}.png", target_frame))?;

    let grad_frame = binary_sdf(&my_frame.add_border(30, 255));

    grad_frame.save_as(&format!("frame_grad_{}.png", target_frame))?;

    let deflated_grad_frame = compress_zlib(&grad_frame.data).unwrap();
    let encoded_deflated_grad_frame_data = general_purpose::STANDARD.encode(&deflated_grad_frame);
    let frame_json = json!(
        {
            "type": "moredfs:single_channel_image_tessellation",
            "x_size": grad_frame.width,
            "z_size": grad_frame.height,
            "deflated_frame_data": encoded_deflated_grad_frame_data
        }
    );

    let frame_json_string = serde_json::to_string_pretty(&frame_json)?;

    fs::write("frame.json", &frame_json_string)?;
    Ok(())
}

fn write_all_frames_to<P>(frames: Vec<MonoFrame>, path: P) -> GResult<()>
where
    P: AsRef<Path>,
{
    fs::create_dir_all(path.as_ref())?;
    let x_size: usize = frames[0].width as usize;
    let z_size: usize = frames[0].height as usize;
    for (index, frame) in (1..).zip(frames.iter()) {
        let grad_frame = binary_sdf(&frame.add_border(32, 255));
        let deflated_grad_frame = compress_zlib(&grad_frame.data).unwrap();
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
            path.as_ref().join(&format!("frame_{}.json", index)),
            &frame_json_string,
        )?;
    }
    write_grid_json(frames.len(), x_size, z_size, path)?;
    Ok(())
}

fn write_grid_json<P>(frame_count: usize, x_size: usize, z_size: usize, path: P) -> GResult<()>
where
    P: AsRef<Path>,
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
    fs::write(path.as_ref().join("all_frames.json"), &frame_json_string)?;
    Ok(())
}

fn compress_zlib(bytes: &[u8]) -> GResult<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes)?;
    Ok(encoder.finish()?)
}

pub struct MonoFrame {
    data: Vec<u8>,
    width: u16,
    height: u16,
}

impl MonoFrame {
    fn new(data: Vec<u8>, width: u16, height: u16) -> MonoFrame {
        MonoFrame {
            data: data,
            width,
            height,
        }
    }

    fn solid_color(width: u16, height: u16, color: u8) -> MonoFrame {
        MonoFrame {
            data: vec![color; width as usize * height as usize],
            width,
            height,
        }
    }

    fn add_border(&self, border_width: u16, border_color: u8) -> MonoFrame {
        let new_width = self.width + 2 * border_width;
        let new_height = self.height + 2 * border_width;

        let mut with_border = MonoFrame::solid_color(new_width, new_height, border_color);

        for y in 0..self.height {
            let src_start = y as usize * self.width as usize;
            let src_end = src_start + self.width as usize;
            let dst_start =
                ((y as usize + border_width as usize) * new_width as usize) + border_width as usize;
            let dst_end = dst_start + self.width as usize;

            with_border.data[dst_start..dst_end].copy_from_slice(&self.data[src_start..src_end]);
        }
        with_border
    }

    fn save_as(&self, filename: &str) -> GResult<()> {
        use image::{ImageBuffer, Luma};

        // Create image buffer from monochromatic data
        let mut img_data = Vec::with_capacity(self.width as usize * self.height as usize);

        // Copy data row by row to handle stride
        for y in 0..self.height {
            let row_start = y as usize * self.width as usize;
            let row_end = row_start as usize + self.width as usize;
            img_data.extend_from_slice(&self.data[row_start..row_end]);
        }

        let img: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_raw(self.width as u32, self.height as u32, img_data)
                .ok_or("Failed to create image buffer")?;

        img.save(filename)?;
        println!("Saved PNG to {}", filename);
        Ok(())
    }
}

fn get_single_channel_frames<P>(video_path: P) -> GResult<Vec<MonoFrame>>
where
    P: AsRef<Path>,
{
    ffmpeg::init()?;

    let mut input = ffmpeg::format::input(video_path.as_ref())?;

    let video_stream = input
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;

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

    for (stream, packet) in input.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut decoded = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
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
    while decoder.receive_frame(&mut decoded).is_ok() {
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

fn binary_sdf(frame: &MonoFrame) -> MonoFrame {
    // First, compute the normal sdf
    let sdf_raw = chebyshev_sdf_two_pass(
        &frame.data,
        frame.width as usize,
        frame.height as usize,
        127, // Splits 0-127 & 128-255
    );

    // Then, find the `max_value` in it
    let max_value = *sdf_raw.iter().max().unwrap();

    // If the `max_value` is 0, then use all white (as the SDF value is inverted)
    if max_value == 0 {
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

fn chebyshev_sdf_two_pass(image: &[u8], width: usize, height: usize, threshold: u8) -> Vec<usize> {
    let mut distance_field: Vec<usize> = vec![usize::MAX; width * height];

    // Sets the distance field value at that position to 0 where the pixel value is above threshold
    distance_field
        .iter_mut()
        .zip(image.iter())
        .for_each(|(dist_val, pixel_val)| {
            if pixel_val > &threshold {
                *dist_val = 0;
            }
        });

    // Forward pass (row-wise, column-wise, diagonal-wise)
    let mut idx = 0;
    for y in 0..height {
        for x in 0..width {
            let mut curr_dist = distance_field[idx];

            // Top-left Diagonal (if within bounds)

            if x > 0 && y > 0 {
                let n_dist = distance_field[idx - width - 1];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist {
                    curr_dist = new_dist;
                }
            }

            // Top (if within bounds)
            if y > 0 {
                let n_dist = distance_field[idx - width];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist {
                    curr_dist = new_dist;
                }
            }

            // Left (if within bounds)
            if x > 0 {
                let n_dist = distance_field[idx - 1];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist {
                    curr_dist = new_dist;
                }
            }

            distance_field[idx] = curr_dist;
            idx += 1;
        }
    }

    // Backward pass (row-wise, column-wise, diagonal-wise)
    distance_field.reverse(); // Better access pattern to reverse all at once and walk forward
    let mut idx = 0;
    for y in 0..height {
        for x in 0..width {
            let mut curr_dist = distance_field[idx];

            // Reversed Top-left diagonal (Bottom-Right Diagonal) (if within bounds)
            if x > 0 && y > 0 {
                let n_dist = distance_field[idx - width - 1];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist {
                    curr_dist = new_dist;
                }
            }

            // Reversed Top (Bottom) (if within bounds)
            if y > 0 {
                let n_dist = distance_field[idx - width];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist {
                    curr_dist = new_dist;
                }
            }

            // Reversed Left (Right) (if within bounds)
            if x > 0 {
                let n_dist = distance_field[idx - 1];
                let new_dist = n_dist + 1;
                if new_dist < curr_dist {
                    curr_dist = new_dist;
                }
            }

            distance_field[idx] = curr_dist;
            idx += 1;
        }
    }
    distance_field.reverse();
    distance_field
}

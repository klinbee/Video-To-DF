use base64::{Engine as _, engine::general_purpose};
use ffmpeg_next as ffmpeg;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;

// Generic Result
type GResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> GResult<()> {
    // let start = Instant::now();
    // let frames = get_single_channel_frames("Bad_Apple!!.mp4")?;
    // println!("{}", frames.len());
    // let duration = start.elapsed();
    // println!("Got all frames in: {}", duration.as_secs_f64());
    //single_frame_test(frames, 1337);
    //write_all_frames_to(frames, "testing/stuff")
    benchmark_real_pipeline()
}

fn benchmark_real_pipeline() -> GResult<()> {
    use std::time::Instant;

    let start = Instant::now();
    fs::create_dir_all("temp")?;
    let frames = get_single_channel_frames("Bad_Apple!!.mp4")?;
    let video_decode_time = start.elapsed();
    println!("Video decode: {:.2}s", video_decode_time.as_secs_f64());

    // 10 Frame Test
    let test_frames: Vec<_> = frames.into_iter().take(10).collect();

    println!("\nTiming per frame (first 10 frames):");
    let x_size = test_frames[0].width as usize + 64;
    let z_size = test_frames[0].height as usize + 64;

    for (index, frame) in test_frames.iter().enumerate() {
        let frame_start = Instant::now();

        // Border
        let border_start = Instant::now();
        let bordered_frame = frame.add_border(32, 255);
        let border_time = border_start.elapsed();

        // SDF
        let sdf_start = Instant::now();
        let grad_frame = binary_sdf(&bordered_frame);
        let sdf_time = sdf_start.elapsed();

        // Compression
        let compress_start = Instant::now();
        let deflated_grad_frame = compress_zlib(&grad_frame)?;
        let compress_time = compress_start.elapsed();

        // Encoding
        let json_start = Instant::now();
        let encoded = general_purpose::STANDARD.encode(&deflated_grad_frame);
        let frame_json = json!({
            "type": "moredfs:single_channel_image_tessellation",
            "x_size": x_size,
            "z_size": z_size,
            "deflated_frame_data": encoded
        });
        let json_string = serde_json::to_string(&frame_json)?;
        let json_time = json_start.elapsed();

        // File Writing
        let write_start = Instant::now();
        fs::write(format!("temp/frame_{}.json", index), &json_string)?;
        let write_time = write_start.elapsed();

        let total_frame_time = frame_start.elapsed();

        println!(
            "Frame {}: Total={:.1}ms | Border={:.1}ms | SDF={:.1}ms | Compress={:.1}ms | JSON={:.1}ms | Write={:.1}ms",
            index,
            total_frame_time.as_millis(),
            border_time.as_millis(),
            sdf_time.as_millis(),
            compress_time.as_millis(),
            json_time.as_millis(),
            write_time.as_millis()
        );
    }

    Ok(())
}

fn single_frame_test(frames: Vec<GrayFrame>, target_frame: usize) -> GResult<()> {
    let my_frame = frames.get(target_frame).ok_or("Frame not found!")?;
    my_frame.save_as(&format!("frame_{}.png", target_frame))?;

    let grad_frame = binary_sdf(&my_frame.add_border(30, 255));

    grad_frame.save_as(&format!("frame_grad_{}.png", target_frame))?;

    let deflated_grad_frame = compress_zlib(&grad_frame).unwrap();
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

fn write_all_frames_to<P>(frames: Vec<GrayFrame>, path: P) -> GResult<()>
where
    P: AsRef<Path>,
{
    fs::create_dir_all(path.as_ref())?;
    let x_size: usize = frames[0].width as usize;
    let z_size: usize = frames[0].height as usize;
    for (index, frame) in frames.iter().enumerate() {
        let grad_frame = binary_sdf(&frame.add_border(32, 255));
        let deflated_grad_frame = compress_zlib(&grad_frame).unwrap();
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

fn compress_zlib(frame: &GrayFrame) -> GResult<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&frame.data)?;
    Ok(encoder.finish()?)
}

pub struct GrayFrame {
    data: Vec<u8>,
    width: u16,
    height: u16,
}

impl GrayFrame {
    fn new(data: Vec<u8>, width: u16, height: u16) -> GrayFrame {
        GrayFrame {
            data: data,
            width,
            height,
        }
    }

    fn solid_color(width: u16, height: u16, color: u8) -> GrayFrame {
        GrayFrame {
            data: vec![color; width as usize * height as usize],
            width,
            height,
        }
    }

    fn add_border(&self, border_width: u16, border_color: u8) -> GrayFrame {
        let new_width = self.width + 2 * border_width;
        let new_height = self.height + 2 * border_width;

        let mut with_border = GrayFrame::solid_color(new_width, new_height, border_color);

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

        // Create image buffer from grayscale data
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

fn get_single_channel_frames<P>(video_path: P) -> GResult<Vec<GrayFrame>>
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

    // Set up context to convert to grayscale
    let mut grayscale_ctx = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::GRAY8, // Single channel grayscale
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )?;

    let mut frames: Vec<GrayFrame> = vec![];

    for (stream, packet) in input.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut decoded = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut gray_frame = ffmpeg::util::frame::video::Video::empty();

                grayscale_ctx.run(&decoded, &mut gray_frame)?;

                frames.push(GrayFrame::new(
                    gray_frame.data(0).to_vec(), // Single channel data
                    gray_frame.width() as u16,
                    gray_frame.height() as u16,
                ));
            }
        }
    }
    // Flush decoder (could be storing extra frames)
    decoder.send_eof()?;
    let mut decoded = ffmpeg::util::frame::video::Video::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let mut gray_frame = ffmpeg::util::frame::video::Video::empty();
        grayscale_ctx.run(&decoded, &mut gray_frame)?;

        frames.push(GrayFrame::new(
            gray_frame.data(0).to_vec(),
            gray_frame.width() as u16,
            gray_frame.height() as u16,
        ));
    }
    Ok(frames)
}

fn binary_sdf(frame: &GrayFrame) -> GrayFrame {
    let sdf_floats = sdf_std(frame);
    let max_value = sdf_floats
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let bytes: Vec<u8> = sdf_floats
        .iter()
        .map(|&f| ((1.0 - (f / max_value)) * 255.0).clamp(0.0, 255.0) as u8)
        .collect();
    GrayFrame::new(bytes, frame.width, frame.height)
}

fn sdf_std(frame: &GrayFrame) -> Vec<f32> {
    sdf_with_threshold(frame, 127)
}

fn sdf_with_threshold(frame: &GrayFrame, threshold: u8) -> Vec<f32> {
    assert_eq!(
        frame.data.len(),
        frame.width as usize * frame.height as usize
    );

    let binary_mask: Vec<bool> = frame.data.iter().map(|&pixel| pixel > threshold).collect();

    let outside_distances = distance_transform(frame, &binary_mask, false);
    let inside_distances = distance_transform(frame, &binary_mask, true);

    let mut sdf = Vec::with_capacity(frame.width as usize * frame.height as usize);

    for i in 0..frame.width as usize * frame.height as usize {
        if binary_mask[i] {
            sdf.push(-inside_distances[i]);
        } else {
            sdf.push(outside_distances[i]);
        }
    }

    sdf
}

fn distance_transform(frame: &GrayFrame, binary_mask: &[bool], invert: bool) -> Vec<f32> {
    let mut distances = vec![f32::INFINITY; frame.width as usize * frame.height as usize];

    // Initialize distances for seed pixels (boundaries)
    for y in 0..frame.height as usize {
        for x in 0..frame.width as usize {
            let idx = y * frame.width as usize + x;
            let is_target = if invert {
                !binary_mask[idx as usize]
            } else {
                binary_mask[idx as usize]
            };

            if is_target {
                distances[idx as usize] = 0.0;
            }
        }
    }

    // Use Danielsson's algorithm for Euclidean distance transform
    danielsson_edt(frame, &mut distances);

    distances
}

/// Danielsson's Euclidean Distance Transform algorithm
fn danielsson_edt(frame: &GrayFrame, distances: &mut [f32]) {
    let w = frame.width as i32;
    let h = frame.height as i32;

    // Forward pass
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;

            if distances[idx] == 0.0 {
                continue;
            }

            let mut min_dist = distances[idx];

            // Check previous neighbors
            for (dy, dx) in &[(-1, -1), (-1, 0), (-1, 1), (0, -1)] {
                let ny = y + dy;
                let nx = x + dx;

                if ny >= 0 && ny < h && nx >= 0 && nx < w {
                    let neighbor_idx = (ny * w + nx) as usize;
                    let neighbor_dist = distances[neighbor_idx];

                    if neighbor_dist != f32::INFINITY {
                        let dist = neighbor_dist + distance_between(0, 0, *dx, *dy);
                        min_dist = min_dist.min(dist);
                    }
                }
            }

            distances[idx] = min_dist;
        }
    }

    // Backward pass
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let idx = (y * w + x) as usize;

            if distances[idx] == 0.0 {
                continue;
            }

            let mut min_dist = distances[idx];

            // Check following neighbors
            for (dy, dx) in &[(0, 1), (1, -1), (1, 0), (1, 1)] {
                let ny = y + dy;
                let nx = x + dx;

                if ny >= 0 && ny < h && nx >= 0 && nx < w {
                    let neighbor_idx = (ny * w + nx) as usize;
                    let neighbor_dist = distances[neighbor_idx];

                    if neighbor_dist != f32::INFINITY {
                        let dist = neighbor_dist + distance_between(0, 0, *dx, *dy);
                        min_dist = min_dist.min(dist);
                    }
                }
            }

            distances[idx] = min_dist;
        }
    }
}

#[inline]
fn distance_between(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    let dx = (x2 - x1) as f32;
    let dy = (y2 - y1) as f32;
    (dx * dx + dy * dy).sqrt()
}

use ffmpeg_next as ffmpeg;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let target_frame: usize = 1337;
    let frames = get_single_channel_frames("Bad_Apple!!.mp4")?;
    let my_frame = frames.get(target_frame).ok_or("Frame not found!")?;
    my_frame.save_as(&format!("frame_{}.png", target_frame))?;
    let duration = start.elapsed();
    println!("Time Elapsed: {}", duration.as_secs_f64());
    Ok(())
}

struct GrayFrame {
    data: Vec<u8>,
    width: u16,
    height: u16,
}

impl GrayFrame {
    fn new(data: &[u8], width: u16, height: u16) -> GrayFrame {
        GrayFrame {
            data: data.to_owned(),
            width,
            height,
        }
    }

    fn solid_color(width: u16, height: u16, color: u8) -> GrayFrame {
        GrayFrame {
            data: vec![color; (width * height) as usize],
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

    fn save_as(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn get_single_channel_frames<P: AsRef<Path>>(
    video_path: P,
) -> Result<Vec<GrayFrame>, Box<dyn std::error::Error>> {
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
                    gray_frame.data(0), // Single channel data
                    gray_frame.width() as u16,
                    gray_frame.height() as u16,
                ));
            }
        }
    }
    Ok(frames)
}

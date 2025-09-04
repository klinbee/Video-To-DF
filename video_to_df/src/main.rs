use ffmpeg_next as ffmpeg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ffmpeg::init()?;

    let mut input = ffmpeg::format::input("Bad_Apple!!.mp4")?;
    let video_stream = input
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;
    let video_stream_index = video_stream.index();

    let context_decoder =
        ffmpeg::codec::context::Context::from_parameters(video_stream.parameters())?;
    let mut decoder = context_decoder.decoder().video()?;

    // Set up scaler to convert to grayscale
    let mut scaler = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::GRAY8, // Single channel grayscale
        decoder.width(),
        decoder.height(),
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )?;

    let mut frame_count = 0;
    let target_frame = 50;

    for (stream, packet) in input.packets() {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet)?;

            let mut decoded = ffmpeg::util::frame::video::Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                frame_count += 1;

                if frame_count == target_frame {
                    // Convert to grayscale
                    let mut gray_frame = ffmpeg::util::frame::video::Video::empty();
                    scaler.run(&decoded, &mut gray_frame)?;

                    // Extract pixel data
                    let width = gray_frame.width() as usize;
                    let height = gray_frame.height() as usize;
                    let data = gray_frame.data(0); // Single channel data
                    let stride = gray_frame.stride(0);

                    println!("Frame {}: {}x{} grayscale", frame_count, width, height);

                    // Save as PNG
                    save_as_png(&data, stride, width, height, "frame_50.png")?;

                    // Save as JPEG
                    save_as_jpeg(&data, stride, width, height, "frame_50.jpg")?;

                    // You can also work with the pixel data directly:
                    process_grayscale_pixels(&data, stride, width, height);

                    return Ok(());
                }
            }
        }
    }

    println!("Video has fewer than {} frames", target_frame);
    Ok(())
}

fn save_as_png(
    data: &[u8],
    stride: usize,
    width: usize,
    height: usize,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Luma};

    // Create image buffer from grayscale data
    let mut img_data = Vec::with_capacity(width * height);

    // Copy data row by row to handle stride
    for y in 0..height {
        let row_start = y * stride;
        let row_end = row_start + width;
        img_data.extend_from_slice(&data[row_start..row_end]);
    }

    let img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width as u32, height as u32, img_data)
            .ok_or("Failed to create image buffer")?;

    img.save(filename)?;
    println!("Saved PNG to {}", filename);
    Ok(())
}

fn save_as_jpeg(
    data: &[u8],
    stride: usize,
    width: usize,
    height: usize,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Luma};

    // Create image buffer from grayscale data
    let mut img_data = Vec::with_capacity(width * height);

    // Copy data row by row to handle stride
    for y in 0..height {
        let row_start = y * stride;
        let row_end = row_start + width;
        img_data.extend_from_slice(&data[row_start..row_end]);
    }

    let img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width as u32, height as u32, img_data)
            .ok_or("Failed to create image buffer")?;

    img.save(filename)?;
    println!("Saved JPEG to {}", filename);
    Ok(())
}

fn process_grayscale_pixels(data: &[u8], stride: usize, width: usize, height: usize) {
    println!("Processing grayscale pixels...");

    // Example: Calculate average brightness
    let mut sum = 0u64;
    let mut pixel_count = 0;

    for y in 0..height {
        for x in 0..width {
            let pixel_value = data[y * stride + x];
            sum += pixel_value as u64;
            pixel_count += 1;
        }
    }

    let average_brightness = sum / pixel_count;
    println!("Average pixel brightness: {}", average_brightness);

    // Example: Find brightest and darkest pixels
    let mut min_val = 255u8;
    let mut max_val = 0u8;

    for y in 0..height {
        for x in 0..width {
            let pixel_value = data[y * stride + x];
            min_val = min_val.min(pixel_value);
            max_val = max_val.max(pixel_value);
        }
    }

    println!("Pixel value range: {} to {}", min_val, max_val);
}

use crate::MonoFrame;

pub fn binary_sdf(frame: &MonoFrame) -> MonoFrame
{
    // Compute the above threshold and below threshold SDF
    // Splits 0-127 & 128-255;
    let above_distances =
        chebyshev_sdf_above(&frame.data, frame.width as usize, frame.height as usize, 127);
    let below_distances =
        chebyshev_sdf_below(&frame.data, frame.width as usize, frame.height as usize, 127);

    // Then, find the `max_value` in them
    let above_max = *above_distances.iter().max().expect("SDF should never have size 0");
    let below_max = *below_distances.iter().max().expect("SDF should never have size 0");

    // Then, convert the `_bytes` from `usize` to `u8` by normalizing to `_max` and clamping
    let above_bytes: Vec<u8> = above_distances
        .iter()
        .map(|&dist| {
            let norm = 1.0 - (dist as f32 / above_max as f32);
            (norm * 127.0).round().clamp(0.0, 127.0) as u8
        })
        .collect();
    let below_bytes: Vec<u8> = below_distances
        .iter()
        .map(|&dist| {
            let norm = dist as f32 / below_max as f32;
            128 + (norm * 127.0).round().clamp(0.0, 127.0) as u8
        })
        .collect();

    // Then, combine them, such that the minimum `below_bytes` masks to `above_bytes`
    let combined_bytes: Vec<u8> = below_bytes
        .iter()
        .zip(&above_bytes)
        .map(|(&below, &above)| {
            match below
            {
                128 => above,
                _ => below,
            }
        })
        .collect();

    // Return it as a MonoFrame
    MonoFrame::new(combined_bytes, frame.width, frame.height)
}

fn chebyshev_sdf_below(
    image: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Vec<usize>
{
    // max distance for chebyshev
    let max_dist = width + height;

    let mut distance_field: Vec<usize> = vec![max_dist; width * height];

    // Sets the distance field value at that position to 0 where the pixel value is above threshold
    distance_field.iter_mut().zip(image.iter()).for_each(|(dist_val, pixel_val)| {
        if pixel_val <= &threshold
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

fn chebyshev_sdf_above(
    image: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
) -> Vec<usize>
{
    // max distance for chebyshev
    let max_dist = width + height;

    let mut distance_field: Vec<usize> = vec![max_dist; width * height];

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
    // Forward pass (right, bottom-right, bottom, bottom-left)
    let mut idx = 0;
    for y in 0..height
    {
        for x in 0..width
        {
            let mut curr_dist = distance_field[idx];

            // Left (if within bounds)
            if x != 0
            {
                curr_dist = curr_dist.min(distance_field[idx - 1] + 1);
            }

            // Top-right Diagonal (if within bounds)
            if (x != (width - 1)) && (y != 0)
            {
                curr_dist = curr_dist.min(distance_field[idx - width + 1] + 1);
            }

            // Top (if within bounds)
            if y != 0
            {
                curr_dist = curr_dist.min(distance_field[idx - width] + 1);
            }

            // Top-left Diagonal (if within bounds)
            if (x != 0) && (y != 0)
            {
                curr_dist = curr_dist.min(distance_field[idx - width - 1] + 1);
            }

            distance_field[idx] = curr_dist;
            idx += 1;
        }
    }
}

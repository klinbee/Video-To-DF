use crate::MonoFrame;

pub fn binary_sdf(frame: &MonoFrame) -> MonoFrame
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

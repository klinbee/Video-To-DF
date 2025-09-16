use std::path::Path;

use image::{
    ImageBuffer,
    Luma,
};

use crate::{
    ImplError,
    Result,
};

pub struct MonoFrame
{
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl MonoFrame
{
    pub fn new(
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

    pub fn solid_color(
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

    pub fn add_border(
        &self,
        border_width: u16,
        border_color: u8,
    ) -> MonoFrame
    {
        let new_width = self.width as usize + 2 * border_width as usize;
        let new_height = self.height as usize + 2 * border_width as usize;

        let mut with_border =
            MonoFrame::solid_color(new_width as u16, new_height as u16, border_color);

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

    pub fn save_as(
        &self,
        filename: &Path,
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
        println!("Saved PNG to {}", filename.display());
        Ok(())
    }
}

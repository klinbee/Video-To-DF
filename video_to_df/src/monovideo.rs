pub struct MonoVideo
{
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl MonoVideo
{
    pub fn new(
        data: Vec<u8>,
        width: u16,
        height: u16,
    ) -> MonoVideo
    {
        MonoVideo {
            data,
            width,
            height,
        }
    }

    pub fn len(&self) -> usize
    {
        self.data.len() / (self.width as usize * self.height as usize)
    }

    pub fn get_frame(
        &self,
        index: usize,
    ) -> Option<&[u8]>
    {
        let start = index * self.frame_size();
        let end = start + self.frame_size();
        self.data.get(start..end)
    }

    fn frame_size(&self) -> usize
    {
        self.width as usize * self.height as usize
    }
}

use std::{
    num::NonZeroU32,
    path::PathBuf,
};

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize)]
pub struct Config
{
    pub video_file: PathBuf,
    pub output_root_dir: PathBuf,
    pub projects: Vec<ProjectConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectConfig
{
    pub border_width: u16,
    pub border_color: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invert_colors: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_start: Option<NonZeroU32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_end: Option<NonZeroU32>,
    pub namespace: String,
    pub make_frames: bool,
    pub frame_dfs_dir: PathBuf,
    pub make_grid: bool,
    pub grid_df_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_frame: Option<NonZeroU32>,
}

impl Default for Config
{
    fn default() -> Self
    {
        Self {
            video_file: PathBuf::from("input.mp4"),
            output_root_dir: PathBuf::from("./output"),
            projects: vec![ProjectConfig::default()],
        }
    }
}

impl Default for ProjectConfig
{
    fn default() -> Self
    {
        Self {
            border_width: 32,
            border_color: 255, // white
            invert_colors: None,
            frame_start: Some(NonZeroU32::new(1).unwrap()),
            frame_end: None,
            namespace: String::from("namespace"),
            make_frames: true,
            frame_dfs_dir: PathBuf::from("./frames"),
            make_grid: true,
            grid_df_dir: PathBuf::from("./"),
            test_frame: Some(NonZeroU32::new(1).unwrap()),
        }
    }
}

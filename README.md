# Video-to-DF
Video to DF (`v2df`) is a Rust CLI tool that lets users convert a video into Minecraft density functions (json-based noise modification functions for terrain) utilizing [More Density Functions](https://github.com/klinbee/More-Density-Functions), a Density Function library mod I made.

Note: Currently not on Cargo, I've personally installed it locally via `cargo install --path .` while in the working directory.

I've used this to create [Bad Apple!! in Minecraft](https://github.com/klinbee/Bad-Apple-World-Preset)

Usage: Type `v2df help` for commands!

Example Config (auto-generated w/ `v2df init`)
```json
{
  "video_file": "input.mp4",
  "output_root_dir": "./home/user/Github/Bad-Apple-World-Preset/Bad_Apple!!_World_Preset/data/bad_apple_world/worldgen/density_function",
  "projects": [
    {
      "border_width": 32,
      "border_color": 255,
      "frame_start": 43,
      "namespace": "bad_apple_world",
      "make_frames": false,
      "frame_dfs_dir": "./frames",
      "make_grid": true,
      "grid_df_dir": "./",
      "make_tp": false,
      "tp_height": 220,
      "tp_dir": "../../functions/frame_tp",
      "test_frame": 1
    },
    {
      "border_width": 32,
      "border_color": 255,
      "frame_start": 1,
      "namespace": "bad_apple_world",
      "make_frames": true,
      "frame_dfs_dir": "./frames",
      "make_grid": true,
      "grid_df_dir": "./video/frames",
      "make_tp": true,
      "tp_height": 220,
      "tp_dir": "../../functions/frame_tp",
      "test_frame": 1
    }
  ]
}

```

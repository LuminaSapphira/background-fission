# background-fission

*Multi-monitor background manager for linux*

---
`background-fission` is a solution for multi-monitor linux users to set different backgrounds on each monitor.
**This project is very new, and as a result only supports the Cinnamon desktop environment.
Feel free to create issues and/or pull requests for other desktop environments.**

**Features:**
- Set a static image or slideshow to each monitor
- Configure monitor resolution and offset
- Periodically changes the slideshow image
- Slideshow period set via a CRON-like syntax

## Building
`background-fission` is a Rust crate, and can be built using `cargo` commands:

`cargo build --release --locked`

## Usage
1. Run `background-fission` once to generate the necessary config file.
    - It will be placed in `$HOME/.config/background-fission/background-fission.json`.
2. Edit the JSON file (documentation below)
3. Optionally run once again to test your configuration
4. Use the [provided systemd unit](https://github.com/CerulanLumina/background-fission/blob/master/dist/background-fission.service) or your own to automatically start `background-fission`.
    - If you want to change the config file, simply do so and restart.

*I recommend using `xrandr` to find the appropriate values for configuration*.
```sh
# Find the total screen size
xrandr | grep -oP '(?<=current )\d+ x \d+'
# 7040 x 1440

# Find each monitor's resolution and offset
$ xrandr | grep -oP '^.* connected (primary )?\d+x\d+\+\d+\+\d+'

# Format: {width}x{height}+{x_offset}+{y_offset}
# HDMI-0 connected 1920x1080+5120+360
# DP-0 connected primary 3440x1440+1680+0
# DP-2 connected 1680x1050+0+390
```

## Configuration

```json5
{
  "width": 7040,   // The total screen width
  "height": 1440,  // The total screen height
  "monitors": [
    {
      "use_slideshow": true,                            // Whether to use a slideshow
      
      "path": "/home/cerulan/Pictures/fhd-bg",          // If use_slideshow is true, the diretory
                                                        // to get images from. If false, the image
                                                        // itself
                                                        
      "width": 1680,                                    // The width of the individual display
      "height": 1050,                                   // The height of the individual display
      "x_offset": 0,                                     // The X Offset of the individual display
      "y_offset": 390                                    // The Y Offset of the individual display
    },
    {
      "use_slideshow": true,
      "path": "/home/cerulan/Pictures/ultrawide-bg",
      "width": 3440,
      "height": 1440,
      "x_offset": 1680,
      "y_offset": 0
    },
    {
      "use_slideshow": true,
      "path": "/home/cerulan/Pictures/fhd-bg",
      "width": 1920,
      "height": 1080,
      "x_offset": 5120,
      "y_offset": 360
    }
  ],
  
  "delay": "0 1/30 * * * * *",                          // The delay (ex: every 30 minutes)
  
  "backend": "Cinnamon"                                 // The backend to use for actually
                                                        // changing the desktop background.
                                                        // Currently only Cinnamon is supported.
}
```

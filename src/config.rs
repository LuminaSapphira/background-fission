use dirs::{config_dir};
use std::fs::{DirBuilder, File, read_dir};
use std::path::PathBuf;
use serde_json::{from_reader, to_writer_pretty};
use rand::seq::IteratorRandom;

/// Holds general configuration information, including monitor configuration and the delay string,
/// in CRON format.
#[derive(Serialize, Deserialize)]
pub struct BFConfig {
    pub width: u32,
    pub height: u32,
    pub monitors: Vec<MonitorConfig>,
    pub delay: String,
    pub backend: Backend
}

/// Holds Monitor-specific configuration information, including the path of backgrounds, the
/// appropriate resolution, and offsets
#[derive(Serialize, Deserialize, Clone)]
pub struct MonitorConfig {
    pub use_slideshow: bool,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
}

#[derive(Serialize, Deserialize)]
pub enum Backend {
    Cinnamon,
    Feh,
}

impl BFConfig {
    /// Loads the background-fission configuration file from the os-specific configuration directory.
    /// Creates the file if it is missing.
    ///
    ///
    /// # Panics
    /// On an IO error, such as unable to read or write the configuration file, or if unable to
    /// parse the configuration file.
    pub fn load() -> BFConfig {
        let config_dir = config_dir().expect("Unable to determine configuration directory")
            .join("background-fission");
        DirBuilder::new()
            .recursive(true)
            .create(&config_dir)
            .expect("Unable to create configuration directory");

        let config_file_path = config_dir.join("background-fission.json");
        if !config_file_path.exists() {

            let config_file = File::create(config_file_path).expect("Unable to create config file");
            let monitors = vec![MonitorConfig{
                use_slideshow: true,
                path: dirs::picture_dir().expect("Getting pictures directory for default").to_str().unwrap().into(),
                width: 1920,
                height: 1080,
                x_offset: 0,
                y_offset: 0
            }];
            let bf_config = BFConfig {
                width: 1920,
                height: 1080,
                monitors,
                delay: String::from("0 1/30 * * * * *"),
                backend: Backend::Feh
            };

            to_writer_pretty(config_file, &bf_config).expect("Unable to write config file");

            bf_config


        } else if config_file_path.is_dir() {
            panic!("Config file already exists as directory");
        } else if config_file_path.is_file() {
            let config_file = File::open(config_file_path).expect("Unable to read config file");
            from_reader(config_file).expect("Unable to parse config file")
        } else {
            panic!("Unknown file issue");
        }
    }

}


impl MonitorConfig {
    /// Gets an image file path from the configuration. If `use_slideshow` is `true`, it will
    /// get a random image from the configured directory. Returns a result that is an error if
    /// there is an IO error when resolving the path or enumerating the directory, and Ok(PathBuf)
    /// otherwise.
    pub fn get_image_path(&self) -> Result<PathBuf, String> {
        let base_path = PathBuf::from(&self.path).canonicalize()
            .map_err(|_| String::from("Unable to canonicalize path"))?;
        if !self.use_slideshow {
            if base_path.is_file() {
                Ok(base_path)
            } else {
                Err(String::from("Path is not file"))
            }
        } else {
            if base_path.is_dir() {
                read_dir(base_path).map_err(|_| String::from("Unable to list directory"))
                    .and_then(|d| {
                        d.filter_map(|res| res.ok())
                            .map(|entry| entry.path())
                            .filter(|path| path.is_file())
                            .choose(&mut rand::thread_rng()).ok_or(String::from("Unable to choose image"))

                    })
            } else {
                Err(String::from("Path is not a directory"))
            }
        }
    }
}
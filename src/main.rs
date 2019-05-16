#[macro_use]
extern crate serde_derive;

use std::thread;
use std::time::{SystemTime, Duration};

use image::{FilterType, DynamicImage};
use image::imageops::replace;
use crate::config::BFConfig;
use job_scheduler::{JobScheduler, Job, Schedule};
use std::sync::{Arc, Mutex};
use std::fs::{DirBuilder};
use std::path::PathBuf;
use thread_priority::{ThreadPriority, ThreadSchedulePolicy, NormalThreadSchedulePolicy};


mod config;
mod backend;

fn main() {
    println!("Starting background-fission");
    println!("Loading configuration file");
    let bfconfig = Arc::new(BFConfig::load());
    let mut sched = JobScheduler::new();
    let parsed_cron: Schedule = bfconfig.delay.parse().expect("Unable to parse cron delay");
    sched.add(Job::new(parsed_cron, || {
        println!("Changing background...");
        let bfconfig = bfconfig.clone();
        let unique = get_epoch();

        let image_dir = dirs::config_dir()
            .expect("Getting config directory")
            .join("background-fission")
            .join("images");

        DirBuilder::new()
            .recursive(true)
            .create(&image_dir)
            .expect("Creating image output directory");

        image_dir.read_dir().expect("Reading image directory for clearing")
            .filter_map(|res| res.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .for_each(|path| std::fs::remove_file(path).expect("Removing old background"));

        make_and_set_background(bfconfig.clone(), unique);



    }));

    make_and_set_background(bfconfig.clone(), get_epoch());

    loop {
        sched.tick();
        thread::sleep(Duration::from_secs(1));
    }

}

fn make_and_set_background(config: Arc<BFConfig>, unique: u64) {
    let output = make_background(config.clone(), unique);
    backend::set_background(&output, &config.backend);
}

trait ReplaceExt {
    fn replace(&mut self, top: &DynamicImage, x_off: u32, y_off: u32);
}

impl ReplaceExt for DynamicImage {
    fn replace(&mut self, top: &DynamicImage, x_off: u32, y_off: u32) {
        replace(self, top, x_off, y_off);
    }
}

fn make_background(config: Arc<BFConfig>, unique: u64) -> PathBuf {
    let buf = Arc::new(Mutex::new(DynamicImage::new_rgba8(config.width, config.height)));

    {
        let mut joins = Vec::new();
        print!("Selected images: [");
        for monitor in &config.monitors {
            let buf = buf.clone();
            let m_cfg = monitor.clone();
            let image_path = m_cfg.get_image_path().expect("Getting image path");
            print!("{:?}, ", image_path.as_os_str());
            let handle = thread::spawn(move || {
                let thread_id = thread_priority::thread_native_id();
                thread_priority::set_thread_priority(thread_id,
                                                     ThreadPriority::Min,
                                                     ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Idle))
                    .expect("Setting thread priority");

                let image = image::open(&image_path)
                    .map_err(|o| (o, format!("Loading image failed: {:?}", &image_path)));
                if image.is_err() {
                    let err = image.as_ref().err().unwrap();
                    eprintln!("{}", err.1);
                    eprintln!("{:?}", err.0);
                }
                let image = image.unwrap_or(DynamicImage::new_rgb8(m_cfg.width, m_cfg.height))
                    .resize_exact(m_cfg.width, m_cfg.height, FilterType::Gaussian);
                {
                    let mut buf = buf.lock().expect("Couldn't acquire lock");
                    buf.replace(&image, m_cfg.x_offset, m_cfg.y_offset);
                }
            });

            joins.push(handle);
        }
        print!("]");
        for join in joins {
            join.join().unwrap_or_else(|_| panic!("Creating image joining"));
        }
    }

    let image_dir = dirs::config_dir()
        .expect("Getting config directory")
        .join("background-fission")
        .join("images");

    DirBuilder::new()
        .recursive(true)
        .create(&image_dir)
        .expect("Creating image output directory");

    let image_path = image_dir.join(format!("background-{}.png", unique));

    buf.lock().expect("Acquiring lock last").save(&image_path).expect("Saving background");

    image_path
}

#[inline]
fn get_epoch() -> u64 {
    SystemTime::UNIX_EPOCH.elapsed().expect("Getting unique time").as_secs()
}
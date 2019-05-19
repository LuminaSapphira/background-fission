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
use std::env::args;
use std::process::exit;


mod config;
mod backend;

fn main() {
    println!("Starting background-fission");
    println!("Loading configuration file");

    let bfconfig = Arc::new(BFConfig::load());

    make_and_set_background(bfconfig.clone(), get_epoch());

    // Cron scheduler
    let mut sched = JobScheduler::new();
    let parsed_cron: Schedule = bfconfig.delay.parse().expect("Unable to parse cron delay");


    if args().len() == 1 {
        // should never fail after check
        if args().next().unwrap().eq("--daemon") {
            println!("Running in daemon mode.");
        } else {
            exit(0);
        }
    } else {
        exit(0);
    }

    // Cron schedule task
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

    loop {
        sched.tick();
        thread::sleep(Duration::from_secs(5));
    }

}

/// Creates the background image, then sets it using the configured backend.
///
/// Takes a unique u64 that will be the temporary file's name
fn make_and_set_background(config: Arc<BFConfig>, unique: u64) {
    let output = make_background(config.clone(), unique);
    backend::set_background(&output, &config.backend);
}

/// Allows for calling the replace function on self without having to pass a mut ref to
/// a global function.
///
/// This lets us get around that `replace` requires a `&mut DynamicImage`, but we will be
/// using a `MutexGuard`.
trait ReplaceExt {
    fn replace(&mut self, top: &DynamicImage, x_off: u32, y_off: u32);
}

impl ReplaceExt for DynamicImage {
    fn replace(&mut self, top: &DynamicImage, x_off: u32, y_off: u32) {
        replace(self, top, x_off, y_off);
    }
}

/// Create the background image file from configuration.
///
/// Takes a unique u64 that will be the temporary file name.
fn make_background(config: Arc<BFConfig>, unique: u64) -> PathBuf {
    // Arc'ed mutex for writing to the image file across threads.
    let buf = Arc::new(
        Mutex::new(
            DynamicImage::new_rgba8(config.width, config.height)
        )
    );

    {
        // New thread per image, so join them when done
        let mut joins = Vec::new();

        print!("Selected images: [");

        // For formatting the stdout
        let len = config.monitors.len();
        let mut num = 0;

        // For each configured monitor, load its image in a new thread and write its data
        // to the big buf image
        for monitor in &config.monitors {

            let buf = buf.clone();

            // Clone so we can move it into the thread
            let m_cfg = monitor.clone();

            // Get the next image's path from config
            let image_path = m_cfg.get_image_path().expect("Getting image path");

            // Print the selected images
            print!("{:?}", image_path.as_os_str());
            num += 1;
            if num < len {
                print!(", ");
            }

            // Spawn a thread for image processing
            let handle = thread::spawn(move || {
                // Set the thread priority low so it doesn't interrupt stuff
                let thread_id = thread_priority::thread_native_id();
                thread_priority::set_thread_priority(thread_id,
                                                     ThreadPriority::Min,
                                                     ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Idle))
                    .expect("Setting thread priority");

                // Load the image
                let image = image::open(&image_path)
                    .map_err(|o| (o, format!("Loading image failed: {:?}", &image_path)));
                if image.is_err() {
                    let err = image.as_ref().err().unwrap();
                    eprintln!("{}", err.1);
                    eprintln!("{:?}", err.0);
                }
                // Resize the image
                let image = image.unwrap_or(DynamicImage::new_rgb8(m_cfg.width, m_cfg.height))
                    .resize_exact(m_cfg.width, m_cfg.height, FilterType::Gaussian);
                // RAII scope for locking the mutex
                {
                    // Lock the mutex and write to the big image
                    let mut buf = buf.lock().expect("Couldn't acquire lock");
                    buf.replace(&image, m_cfg.x_offset, m_cfg.y_offset);
                }
            });

            joins.push(handle);
        }
        // End of image list
        println!("]");

        // Join all the threads
        for join in joins {
            join.join().unwrap_or_else(|_| panic!("Creating image joining"));
        }
    }

    // Output the new big image into the temp image folder
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

/// Gets the unix epoch in seconds
/// # Panics
/// When the elapsed time was unavailable (is this even possible?)
#[inline]
fn get_epoch() -> u64 {
    SystemTime::UNIX_EPOCH.elapsed().expect("Getting unique time").as_secs()
}
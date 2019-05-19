use crate::config::Backend;
use std::path::PathBuf;
use std::process::Command;


/// Set the background using a specified backend
pub fn set_background(path: &PathBuf, backend: &Backend) {
    match backend {
        Backend::Cinnamon => {
            println!("Setting background via Cinnamon");
            let result = path.as_os_str().to_str()
                .ok_or(String::from("Unable to set background -- converting string"))
                .and_then(|strpath| {
                    println!("Setting picture-uri: \"'file://{}'\"", strpath);
                    dconf_rs::set_string(
                        "/org/cinnamon/desktop/background/picture-uri",
                        format!("file://{}", strpath).as_str()
                    )
                })
                .and_then(|_| {
                    dconf_rs::set_string(
                        "/org/cinnamon/desktop/background/picture-options",
                        "spanned"
                    )
                });
            if result.is_err() {
                eprintln!("Unable to set background: {:?}", result.unwrap_err());
            } else {
                result.unwrap();
            }

        },
        Backend::Feh => {
            println!("Setting background using feh");
            let result = path.as_os_str().to_str()
                .ok_or(String::from("Unable to set background -- converting string"))
                .and_then(|strpath| {
                    Command::new("feh")
                        .args(&["--no-xinerama", "--bg-center", strpath])
                        .output()
                        .map_err(|_| String::from("Unable to set background -- using feh"))
                })
                .and(Ok(()));
            if result.is_err() {
                eprintln!("Unable to set background: {:?}", result.unwrap_err());
            } else {
                result.unwrap();
            }

        }
    }
}
use crate::config::Backend;
use std::path::PathBuf;

pub fn set_background(path: &PathBuf, backend: &Backend) {
    match backend {
        Backend::Cinnamon => {
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

        }
    }
}
use std::io::{self};
use std::path::Path;
use std::fs::File;

enum OverwriteOrSkip {
    Overwrite,
    Skip,
}

pub enum OverwriteConfiguration {
    Ask,
    OverwriteAll,
    SkipAll,
}

pub struct FileCreator {
    pub overwrite_mode_config: OverwriteConfiguration,
}

impl FileCreator {

    pub fn create_file(&mut self, filename: &str) -> Option<File> {
        let file_exists = Path::new(&filename).exists();    
     
        if file_exists {
            let overwrite_mode = &self.overwrite_handling(&filename);
            match overwrite_mode {
                OverwriteOrSkip::Skip => return None,
                _ => {}
            }
        }
    
        let file = File::create(filename);
        match file {
            Ok(f) => return Some(f),
            Err(e) => {
                println!("encountered error when trying to create file {}", filename);
                println!("error: {}", e);
                return None;
            }
        }
    }

    fn overwrite_handling(&mut self, filename: &str) -> OverwriteOrSkip {
        match &self.overwrite_mode_config {
            OverwriteConfiguration::OverwriteAll => return OverwriteOrSkip::Overwrite,
            OverwriteConfiguration::SkipAll => return OverwriteOrSkip::Skip,
            _ => {}
        }

        let mut buffer = String::new();
        loop {
            println!("WARNING! osm_extract_polygon wanted to create the file {}, but it exists already. [s]kip, [o]verwrite, s[k]ip all, overwrite [a]ll?", filename);

            io::stdin().read_line(&mut buffer).expect("Couldn't read line");

            buffer = String::from(buffer.trim());

            if buffer.as_str() == "k" {
                self.overwrite_mode_config = OverwriteConfiguration::SkipAll;
            } else if buffer.as_str() == "a" {
                self.overwrite_mode_config = OverwriteConfiguration::OverwriteAll;
            }

            match buffer.as_str() {
                "s" | "k" => return OverwriteOrSkip::Skip,
                "o" | "a" => return OverwriteOrSkip::Overwrite,
                _ => {
                    buffer = String::from("");
                }
            }
        }
    }
}

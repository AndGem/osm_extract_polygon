use std::fs::File;
use std::io::{self};
use std::path::Path;

use crate::output::OverwriteConfiguration;

use std::io::{Error, ErrorKind};

pub struct FileCreator {
    pub overwrite_mode_config: OverwriteConfiguration,
}

enum OverwriteOrSkip {
    Overwrite,
    Skip,
}

impl FileCreator {
    pub fn create_file(&mut self, filename: &str) -> std::io::Result<File> {
        if Path::new(filename).exists() {
            match self.overwrite_handling(filename)? {
                OverwriteOrSkip::Skip => {
                    return Err(Error::new(ErrorKind::AlreadyExists, "skipped"));
                }
                OverwriteOrSkip::Overwrite => {}
            }
        }

        File::create(filename)
    }

    fn overwrite_handling(&mut self, filename: &str) -> io::Result<OverwriteOrSkip> {
        match self.overwrite_mode_config {
            OverwriteConfiguration::OverwriteAll => return Ok(OverwriteOrSkip::Overwrite),
            OverwriteConfiguration::SkipAll => return Ok(OverwriteOrSkip::Skip),
            _ => {}
        }

        let mut buffer = String::new();
        loop {
            println!("WARNING! osm_extract_polygon wanted to create the file {}, but it exists already. [s]kip, [o]verwrite, s[k]ip all, overwrite [a]ll?", filename);

            io::stdin().read_line(&mut buffer).expect("Couldn't read line");

            let input = buffer.trim();

            match input {
                "s" => return Ok(OverwriteOrSkip::Skip),
                "o" => return Ok(OverwriteOrSkip::Overwrite),
                "k" => {
                    self.overwrite_mode_config = OverwriteConfiguration::SkipAll;
                    return Ok(OverwriteOrSkip::Skip);
                }
                "a" => {
                    self.overwrite_mode_config = OverwriteConfiguration::OverwriteAll;
                    return Ok(OverwriteOrSkip::Overwrite);
                }
                _ => {
                    buffer.clear();
                }
            }
        }
    }
}

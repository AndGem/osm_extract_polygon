use crate::converter::Polygon;
use crate::output::output_handler::FileWriter;

use std::fs::File;
use std::io::prelude::*;

pub struct PolyWriter {}

impl FileWriter for PolyWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()> {
        file.write_all(&polygon.name.as_bytes())?;
        file.write_all(b"\n")?;
        let mut index: i32 = 1;
        for points in &polygon.points {
            file.write_fmt(format_args!("area_{}\n", index))?;
            for point in points {
                file.write_fmt(format_args!("\t{} \t{}\n", point.lon, point.lat))?;
            }
            file.write_all(b"END\n")?;
            index += 1;
        }
        file.write_all(b"END\n")?;
        Ok(())
    }
}

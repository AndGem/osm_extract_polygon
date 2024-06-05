use crate::converter::Polygon;
use crate::output::output_handler::FileWriter;

use std::fs::File;
use std::io::prelude::*;

pub struct PolyWriter {}

impl FileWriter for PolyWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()> {
        let mut output: String = String::new();
        output.push_str(&polygon.name);
        output.push('\n');

        for (index, points) in polygon.points.iter().enumerate() {
            let area_id_str = format!("area_{}\n", index + 1);
            output.push_str(&area_id_str);

            for point in points {
                let point_str = format!("\t{} \t{}\n", point.lon, point.lat);
                output.push_str(&point_str);
            }
            
            output.push_str("END\n");
        }
        output.push_str("END\n");
        file.write_all(output.as_bytes())?;

        Ok(())
    }
}

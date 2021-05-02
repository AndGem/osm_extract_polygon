use crate::converter::Polygon;
use crate::output::output_handler::FileWriter;

use std::fs::File;

pub struct GeoJsonWriter {}

impl FileWriter for GeoJsonWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()> {
        Ok(())
        //TODO: fill in code to write geojson
    }
}

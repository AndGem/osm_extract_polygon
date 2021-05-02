pub mod output_handler;

mod file_creator;
mod file_writer_geojson;
mod file_writer_poly;

pub enum OverwriteConfiguration {
    Ask,
    OverwriteAll,
    SkipAll,
}

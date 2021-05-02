pub mod file_writer_geojson;
pub mod file_writer_poly;
pub mod output_handler;

mod file_creator;

pub enum OverwriteConfiguration {
    Ask,
    OverwriteAll,
    SkipAll,
}

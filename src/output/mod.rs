pub mod output_handler;
pub mod writer_geojson;
pub mod writer_poly;

mod file_creator;

pub enum OverwriteConfiguration {
    Ask,
    OverwriteAll,
    SkipAll,
}

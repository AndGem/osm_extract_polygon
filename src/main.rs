extern crate osmpbfreader;

mod converter;
mod osm_reader;
mod poly_writer;

fn main() {

    let in_filename = "karlsruhe-regbez-latest.osm.pbf";

    let relations = osm_reader::read_osm(in_filename);
    let polygons = converter::convert(relations);
    poly_writer::write(&".".to_string(), &polygons);
    println!("{:?}", polygons);
}

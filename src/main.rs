extern crate osmpbfreader;

mod converter;
mod osm_reader;
mod poly_writer;

fn main() {

    let in_filename = "karlsruhe-regbez-latest.osm.pbf";

    let relations = osm_reader::read_osm(in_filename);
    let polygons = converter::convert(relations);
    
    let result = poly_writer::write(&"./output".to_string(), &polygons);

    match result {
        Ok(_) => println!("success!"),
        Err(_) => println!("error!"),
    }
}

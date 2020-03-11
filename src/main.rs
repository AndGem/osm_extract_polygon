extern crate osmpbfreader;

mod converter;
mod osm_reader;
mod poly_writer;

fn main() {

    //TODO: argument parsing
    //TODO: write proper README
    //TODO: 

    // let in_filename = "karlsruhe-regbez-latest.osm.pbf";
    // let in_filename = "spain-latest.osm.pbf";
    let in_filename = "iceland-latest.osm.pbf";

    let relations = osm_reader::read_osm(in_filename);
    let polygons = converter::convert(relations);
    
    let path = format!("{}_polygons", in_filename);
    let result = poly_writer::write(&path, &polygons);

    match result {
        Ok(size) => println!("success! wrote {} files!", size),
        Err(e) => println!("error! {:?}", e),
    }
}

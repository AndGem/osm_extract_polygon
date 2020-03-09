extern crate osmpbfreader;

mod osm_reader;


fn main() {

    //process
    let in_filename = "karlsruhe-regbez-latest.osm.pbf";
    // let in_filename = "berlin-latest.osm.pbf";
    // let (nodes, ways) = osm_reader::read_osm(&in_filename.to_owned(), &config);
    // let graph = osm_convert::convert(nodes, ways, &config);
    osm_reader::read_osm(in_filename);
}

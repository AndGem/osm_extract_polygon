use crate::converter::Polygon;

use std::fs::{create_dir_all, File};
use std::io::prelude::*;

pub fn write(folder: &str, polygons: &[Polygon]) -> std::io::Result<usize> {
    let _create_result = create_dir_all(folder);

    // Array of length counting how many times this region name is duplicated (see: https://github.com/AndGem/osm_extract_polygon/issues/10)
    let dupnamecount = polygons.map(|v|  0 ); // Initialize as all-zeros with same length as polygons.len()

    // For each region name, count how many times we find it in the list of all regions
    for i in 1..dupnamecount.len() {
      dupnamecount[i] = polygons.iter().filter(|&p| *p == polygons[i]).count(); // Does "==" do string comparison in Rust?
    }

    for pi = 0..polygons.len() { // Changed, since we need to be certain that the iteration proceeds in sync over polygons[] and dupnamecount[]
        let polygon = polygons[pi];
        let name: String = make_safe(&polygon.name);

        // Handle potential duplicated region names
        // The bug is described here: https://github.com/AndGem/osm_extract_polygon/issues/10
        // In the normal case of a unique region name this will be decremented from 1 to 0 here
        // In the case of, say, 3 regions with the same name, this will be decremented from 3 to 2 here, 
        // and the filename will be suffixed in descending order with "_2", "_1","_0" suffixes to ensure unique filenames.
        dupnamecount[pi]--;
        let filename = (dupnamecount[0]>0) ? format!("{}/{}_{}.poly", folder, name, dupnamecount[pi]) : format!("{}/{}.poly", folder, name)

        println!("{}", filename);
        let mut file = File::create(filename)?;
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
    }

    Ok(polygons.len())
}

//TODO: this is probably not sufficient to be really safe
fn make_safe(name: &str) -> String {
    name.replace("\\", "")
        .replace("/", "")
        .replace("&", "")
        .replace(":", "")
        .replace("<", "")
        .replace(">", "")
}

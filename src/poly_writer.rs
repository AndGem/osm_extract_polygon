use crate::converter::Polygon;

use std::fs::{create_dir_all, File};
use std::io::prelude::*;

pub fn write(folder: &str, polygons: &[Polygon]) -> std::io::Result<usize> {
    let _create_result = create_dir_all(folder);

    for polygon in polygons {
        let name: String = make_safe(&polygon.name);
        let filename = format!("{}/{}.poly", folder, name);
        println!("{}", filename);
        let mut file = File::create(filename)?;
        file.write_all(&polygon.name.as_bytes())?;
        file.write_all(b"\n")?;

        let mut index: i32 = 1;
        for points in &polygon.points {
            file.write_fmt(format_args!("area_{}\n", index))?;
            for point in points {
                file.write_fmt(format_args!("\t{},\t{}\n", point.lon, point.lat))?;
            }

            file.write_all(b"END\n")?;
            index += 1;
        }
    }

    Ok(polygons.len())
}

fn make_safe(name: &str) -> String {
    name.replace("\\", "")
        .replace("/", "")
        .replace("&", "")
        .replace(":", "")
        .replace("<", "")
        .replace(">", "")
}

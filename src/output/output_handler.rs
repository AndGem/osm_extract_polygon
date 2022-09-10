use crate::converter::Polygon;
use crate::output::file_creator::FileCreator;
use crate::output::file_writer_geojson::GeoJsonWriter;
use crate::output::file_writer_poly::PolyWriter;
use crate::output::OverwriteConfiguration;

use std::fs::File;
use std::time::Instant;

use std::collections::HashSet;
use std::fs::create_dir_all;

pub trait FileWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()>;
}

pub struct OutputHandlerConfiguration {
    pub overwrite_configuration: OverwriteConfiguration,
    pub geojson_output: bool,
}

pub fn write(folder: &str, polygons: &[Polygon], config: OutputHandlerConfiguration) -> std::io::Result<u64> {
    let _create_result = create_dir_all(folder)?;

    let filename_polys = pair_safe_filenames_and_polygons(polygons);

    let mut output_handler = new_output_handler(config);

    output_handler.write_files(folder, filename_polys)
}

fn new_output_handler(config: OutputHandlerConfiguration) -> OutputHandler {
    OutputHandler {
        file_creator: FileCreator {
            overwrite_mode_config: config.overwrite_configuration,
        },
        write_poly: true,
        write_geojson: config.geojson_output,
    }
}

struct OutputHandler {
    file_creator: FileCreator,
    write_poly: bool,
    write_geojson: bool,
}

impl OutputHandler {
    pub fn write_files(&mut self, base_folder: &str, filename_polys: Vec<(String, &Polygon)>) -> std::io::Result<u64> {
        let mut file_count: u64 = 0;

        let poly_writer = PolyWriter {};
        let geojson_writer = GeoJsonWriter {};

        let now = Instant::now();
        println!("writing output files...");

        for (name, polygon) in filename_polys {
            let filename_wo_ext = format!("{}/{}", base_folder, name);
            if self.write_poly {
                let success_poly = self.write_file(&filename_wo_ext, "poly", polygon, &poly_writer);
                if success_poly {
                    file_count += 1;
                }
            }
            if self.write_geojson {
                let success_geojson = self.write_file(&filename_wo_ext, "geojson", polygon, &geojson_writer);
                if success_geojson {
                    file_count += 1;
                }
            }
        }

        println!("finished writing! {}s", now.elapsed().as_secs());

        Ok(file_count)
    }

    pub fn write_file(
        &mut self,
        filename_wo_ext: &str,
        ext: &str,
        polygon: &Polygon,
        file_writer: &impl FileWriter,
    ) -> bool {
        let filename = format!("{}.{}", filename_wo_ext, ext);

        let result = self
            .file_creator
            .create_file(&filename)
            .and_then(|mut file| file_writer.write_to_file(&mut file, polygon));

        match result {
            Err(e) => {
                println!("{}: {}", filename, e);
                false
            }
            Ok(_) => {
                println!("{}: successfully written ", filename);
                true
            }
        }
    }
}

fn pair_safe_filenames_and_polygons(polygons: &[Polygon]) -> Vec<(String, &Polygon)> {
    let safe_names: Vec<String> = polygons.iter().map(|p| make_safe(&p.name)).collect();

    let mut seen_names: HashSet<String> = HashSet::new();
    let mut duplicate_names: HashSet<String> = HashSet::new();

    safe_names.iter().for_each(|name| {
        if seen_names.contains(&name.to_lowercase()) {
            duplicate_names.insert(name.to_string().to_lowercase());
        } else {
            seen_names.insert(name.to_string().to_lowercase());
        }
    });

    safe_names
        .iter()
        .zip(polygons.iter())
        .map(|(name, p)| {
            let out_name = if duplicate_names.contains(&name.to_lowercase()) {
                format!("{}_{}", name, p.relation_id)
            } else {
                name.to_string()
            };

            (out_name, p)
        })
        .collect()
}

fn make_safe(name: &str) -> String {
    let mut s = String::from(name);
    s.retain(|c| !r#"\\/&:<>|*"#.contains(c));
    s
}

// ////////////////////////////////////
// ////////////////////////////////////
// UNIT TESTS
// ////////////////////////////////////
// ////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_safe_remvoes_forbidden_chars() {
        let input = "abc&:<>/\\|*";
        let expected = "abc";

        let result = make_safe(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_make_safe_doesnt_replace_harmless_characters() {
        let input = "jhdsakljvsjkasspasd";
        let expected = "jhdsakljvsjkasspasd";

        let result = make_safe(input);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_filenames_add_extensions_to_duplicate_regions() {
        let p1_name = String::from("spain_region");
        let p1_name_clone = p1_name.clone();
        let p1_name_clone2 = p1_name.clone();
        let p2_name = String::from("french_region");

        let expected = [
            p1_name.clone() + "_100",
            p2_name.clone(),
            p1_name.clone() + "_300",
            p1_name.clone() + "_400",
        ]
        .to_vec();

        let p1 = Polygon {
            name: p1_name,
            points: Vec::new(),
            relation_id: 100,
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
            relation_id: 200,
        };

        let p3 = Polygon {
            name: p1_name_clone,
            points: Vec::new(),
            relation_id: 300,
        };

        let p4 = Polygon {
            name: p1_name_clone2,
            points: Vec::new(),
            relation_id: 400,
        };

        let input = [p1, p2, p3, p4];

        let result = pair_safe_filenames_and_polygons(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }

    #[test]
    fn test_create_filenames_add_no_extensions_if_unique() {
        let p1_name = String::from("spanish_region");
        let p2_name = String::from("french_region");
        let p3_name = String::from("german_region");

        let expected = [p1_name.clone(), p2_name.clone(), p3_name.clone()];

        let p1 = Polygon {
            name: p1_name,
            points: Vec::new(),
            relation_id: 1,
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
            relation_id: 2,
        };

        let p3 = Polygon {
            name: p3_name,
            points: Vec::new(),
            relation_id: 3,
        };

        let input = [p1, p2, p3];

        let result = pair_safe_filenames_and_polygons(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }

    #[test]
    fn test_create_filenames_ignores_case_for_duplicates_but_retains_original() {
        let p1_name = String::from("spanish_region");
        let p2_name = String::from("SPAniSh_RegION");

        let expected = [p1_name.clone() + "_123", p2_name.clone() + "_456"];

        let p1 = Polygon {
            name: p1_name,
            points: Vec::new(),
            relation_id: 123,
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
            relation_id: 456,
        };

        let input = [p1, p2];

        let result = pair_safe_filenames_and_polygons(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }
}

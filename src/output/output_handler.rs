use crate::converter::Polygon;
use crate::output::file_creator::FileCreator;
use crate::output::file_writer_geojson::GeoJsonWriter;
use crate::output::file_writer_poly::PolyWriter;
use crate::output::OverwriteConfiguration;

use std::fs::File;

use std::collections::HashMap;
use std::fs::create_dir_all;

pub trait FileWriter {
    fn write_to_file(&self, file: &mut File, polygon: &Polygon) -> std::io::Result<()>;
}

pub fn write(
    folder: &str,
    polygons: &[Polygon],
    overwrite_configuration: OverwriteConfiguration,
) -> std::io::Result<u64> {
    let _create_result = create_dir_all(folder)?;

    let filename_polys = pair_safe_filenames_and_polygons(polygons);

    let mut output_handler = new_output_hanlder(
        FileCreator {
            overwrite_mode_config: overwrite_configuration,
        },
        true,
    );

    output_handler.write_files(folder, filename_polys)
}

fn new_output_hanlder(file_creator: FileCreator, write_geojson: bool) -> OutputHandler {
    OutputHandler {
        file_creator,
        write_poly: true,
        write_geojson,
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

    let mut duplicate_count: HashMap<String, usize> = count_duplicate_names(&safe_names);

    safe_names
        .iter()
        .zip(polygons.iter())
        .map(|(name, p)| {
            let out_name;
            if duplicate_count.contains_key(&name.to_lowercase()) {
                let val = duplicate_count.get_mut(&name.to_lowercase()).unwrap();
                out_name = format!("{}_{}", name, val);
                *val -= 1;
            } else {
                out_name = name.to_string();
            }
            (out_name, p)
        })
        .collect()
}

fn make_safe(name: &str) -> String {
    let mut s = String::from(name);
    s.retain(|c| !r#"\\/&:<>|*"#.contains(c));
    s
}

fn count_duplicate_names(safe_names: &[String]) -> HashMap<String, usize> {
    let mut m: HashMap<String, usize> = HashMap::new();
    for x in safe_names {
        *m.entry(x.to_string().to_lowercase()).or_default() += 1;
    }

    m.into_iter().filter(|&(_, v)| v != 1).collect()
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
    fn test_count_duplicates_when_input_is_unique_return_empty_hashmap() {
        let p1_name = String::from("abc123");
        let p2_name = String::from("defgh1");
        let p3_name = String::from("aaaddd");

        let input = [p1_name, p2_name, p3_name];
        let result = count_duplicate_names(&input);

        assert_eq!(result, HashMap::new());
    }

    #[test]
    fn test_count_duplicates_when_input_contains_duplicates_then_have_them_in_hashmap() {
        let p1_name = String::from("random_name");
        let p1_name_copy = p1_name.clone();
        let p2_name = String::from("random_name2");

        let expected: HashMap<String, usize> = [(p1_name.clone(), 2)].iter().cloned().collect();

        let input = [p1_name, p2_name, p1_name_copy];

        let result = count_duplicate_names(&input);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_count_duplicates_when_input_contains_duplicates_then_have_them_in_hashmap_and_ignores_case() {
        let p1_name = String::from("random_name");
        let p1_name_copy = String::from("RandOm_NAme");
        let p2_name = String::from("random_name2");

        let expected: HashMap<String, usize> = [(p1_name.clone(), 2)].iter().cloned().collect();

        let input = [p1_name, p2_name, p1_name_copy];

        let result = count_duplicate_names(&input);

        assert_eq!(result, expected);
    }
    #[test]
    fn test_create_filenames_add_extensions_to_duplicate_regions() {
        let p1_name = String::from("spain_region");
        let p1_name_clone = p1_name.clone();
        let p1_name_clone2 = p1_name.clone();
        let p2_name = String::from("french_region");

        let expected = [
            p1_name.clone() + "_3",
            p2_name.clone(),
            p1_name.clone() + "_2",
            p1_name.clone() + "_1",
        ]
        .to_vec();

        let p1 = Polygon {
            name: p1_name,
            points: Vec::new(),
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
        };

        let p3 = Polygon {
            name: p1_name_clone,
            points: Vec::new(),
        };

        let p4 = Polygon {
            name: p1_name_clone2,
            points: Vec::new(),
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
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
        };

        let p3 = Polygon {
            name: p3_name,
            points: Vec::new(),
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

        let expected = [p1_name.clone() + "_2", p2_name.clone() + "_1"];

        let p1 = Polygon {
            name: p1_name,
            points: Vec::new(),
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
        };

        let input = [p1, p2];

        let result = pair_safe_filenames_and_polygons(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }
}

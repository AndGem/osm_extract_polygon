use crate::converter::Polygon;

use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::io::{self};
use std::path::Path;

#[derive(PartialEq, Clone)]
enum ConflictMode {
    Ask,
    OverwriteAll,
    SkipAll,
    Skip,
    Overwrite,
}

pub fn write(folder: &str, polygons: &[Polygon]) -> std::io::Result<usize> {
    let _create_result = create_dir_all(folder);

    let filename_polys = create_filenames(polygons);

    let mut conflict_mode: ConflictMode = ConflictMode::Ask;

    let mut file_count: usize = 0;

    for (name, polygon) in filename_polys {
        let filename = format!("{}/{}.poly", folder, name);
        println!("{}", filename);

        let file_exists = Path::new(&filename).exists();
        if file_exists {
            conflict_mode = overwrite_handling(&filename, conflict_mode);
            match conflict_mode {
                ConflictMode::Skip => continue,
                ConflictMode::SkipAll => {
                    println!("... skipping");
                    continue;
                }
                ConflictMode::OverwriteAll => {}
                ConflictMode::Overwrite => {}
                _ => {}
            }
        }

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

        file_count += 1;
    }

    Ok(file_count)
}

fn overwrite_handling(filename: &str, conflict_mode: ConflictMode) -> ConflictMode {
    if conflict_mode == ConflictMode::OverwriteAll || conflict_mode == ConflictMode::SkipAll {
        return conflict_mode;
    }

    let mut buffer = String::new();
    loop {
        println!("WARNING! osm_extract_polygon wanted to create the file {}, but it exists already. [s]kip, [o]verwrite, s[k]ip all, overwrite [a]ll?", filename);

        io::stdin().read_line(&mut buffer).expect("Couldn't read line");

        buffer = String::from(buffer.trim());

        match buffer.as_str() {
            "s" => return ConflictMode::Skip,
            "o" => return ConflictMode::Overwrite,
            "k" => return ConflictMode::SkipAll,
            "a" => return ConflictMode::OverwriteAll,
            _ => {
                buffer = String::from("");
            }
        }
    }
}

fn create_filenames(polygons: &[Polygon]) -> Vec<(String, &Polygon)> {
    let safe_names: Vec<String> = polygons
        .iter()
        .map(|p| make_safe(&p.name))
        .map(|name| name.to_lowercase())
        .collect();

    let mut duplicate_count: HashMap<String, usize> = count_duplicate_names(&safe_names);

    safe_names
        .iter()
        .zip(polygons.iter())
        .map(|(name, p)| {
            let out_name;
            if duplicate_count.contains_key(name) {
                let val = duplicate_count.get_mut(name).unwrap();
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
        *m.entry(x.to_string()).or_default() += 1;
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

        let result = create_filenames(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }

    #[test]
    fn test_create_filenames_add_no_extensions_if_unique() {
        let p1_name = String::from("spain_region");
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

        let result = create_filenames(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }

    #[test]
    fn test_create_filenames_ignores_cases() {
        let p1_name = String::from("spain_region");
        let p2_name = String::from("SPAin_RegION");

        let expected = [p1_name.clone() + "_2", p2_name.clone().to_lowercase() + "_1"];

        let p1 = Polygon {
            name: p1_name,
            points: Vec::new(),
        };

        let p2 = Polygon {
            name: p2_name,
            points: Vec::new(),
        };

        let input = [p1, p2];

        let result = create_filenames(&input);

        let result_names: Vec<String> = result.iter().map(|(x, _y)| x).cloned().collect();

        assert_eq!(result_names, expected);
    }
}

use osmpbfreader::{Node, NodeId, OsmPbfReader, Relation, RelationId, WayId};

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::i8::MAX;
use std::path::Path;
use std::time::Instant;

use crate::utils::values_to_set;

type OsmPbfReaderFile = osmpbfreader::OsmPbfReader<std::fs::File>;

#[derive(Clone)]
pub struct RelationNodes {
    pub relation: Relation,
    pub nodes: Vec<Vec<Node>>,
}

pub fn read_osm(filename: &str, min_admin: &i8, max_admin: &i8) -> Result<Vec<RelationNodes>, std::io::Error> {
    let file = File::open(Path::new(filename))?;
    read_ways_and_relation(file, min_admin, max_admin)
}

fn read_ways_and_relation(file: File, min_admin: &i8, max_admin: &i8) -> Result<Vec<RelationNodes>, std::io::Error> {
    let mut reader = OsmPbfReader::new(file);

    let relations = find_admin_boundary_relations(&mut reader, min_admin, max_admin);

    let relation_to_ways = find_ways_for_relation_ids(&relations);
    let way_to_nodes = find_nodes_for_way_ids(&mut reader, values_to_set(&relation_to_ways));
    let node_map = find_nodes_for_node_ids(&mut reader, values_to_set(&way_to_nodes));

    let relation_to_nodes = relation_to_ways
        .iter()
        .map(|(r_id, ways)| {
            let nodes = replace_way_id_with_node_ids(ways, &way_to_nodes);
            let nodes = replace_node_id_with_node(nodes, &node_map);
            let relation = relations.get(r_id).expect("Relation not found").clone();
            RelationNodes { relation, nodes }
        })
        .collect();

    Ok(relation_to_nodes)
}

fn has_proper_admin_level(relation: &Relation, min_admin: &i8, max_admin: &i8) -> bool {
    let admin_level: i8 = relation
        .tags
        .get("admin_level")
        .and_then(|v| v.parse::<i8>().ok())
        .unwrap_or(MAX);

    (*min_admin <= admin_level) && (admin_level <= *max_admin)
}

fn extract_way_ids_from_relation(relation: &Relation) -> Vec<WayId> {
    relation.refs.iter().filter_map(|r| r.member.way()).collect()
}

fn replace_way_id_with_node_ids(
    way_ids: &[WayId],
    way_id_to_node_ids: &HashMap<WayId, Vec<NodeId>>,
) -> Vec<Vec<NodeId>> {
    way_ids
        .iter()
        .filter_map(|way_id| way_id_to_node_ids.get(way_id))
        .cloned()
        .collect()
}

fn replace_node_id_with_node(v_node_ids: Vec<Vec<NodeId>>, node_id_to_node: &HashMap<NodeId, Node>) -> Vec<Vec<Node>> {
    v_node_ids
        .iter()
        .map(|node_ids| {
            node_ids
                .iter()
                .filter_map(|node_id| node_id_to_node.get(node_id))
                .cloned()
                .collect()
        })
        .collect()
}

fn find_admin_boundary_relations(
    pbf: &mut OsmPbfReaderFile,
    min_admin: &i8,
    max_admin: &i8,
) -> HashMap<RelationId, Relation> {
    let now = Instant::now();
    println!("parsing relations...");

    let relation_id_to_relation: HashMap<RelationId, Relation> = pbf
        .par_iter()
        .map(Result::unwrap)
        .filter(|obj| obj.is_relation())
        .filter(|obj| obj.relation().unwrap().tags.contains("boundary", "administrative"))
        .filter(|obj| has_proper_admin_level(obj.relation().unwrap(), min_admin, max_admin))
        .map(|obj| obj.relation().unwrap().clone())
        .map(|relation| (relation.id, relation))
        .collect();

    println!(
        "finished parsing {} relations! {}s",
        relation_id_to_relation.len(),
        now.elapsed().as_secs()
    );
    relation_id_to_relation
}

fn find_ways_for_relation_ids(
    relation_id_to_relation: &HashMap<RelationId, Relation>,
) -> HashMap<RelationId, Vec<WayId>> {
    relation_id_to_relation
        .iter()
        .map(|(relation_id, relation)| (*relation_id, extract_way_ids_from_relation(relation)))
        .collect()
}

fn find_nodes_for_way_ids(pbf: &mut OsmPbfReaderFile, way_ids: HashSet<WayId>) -> HashMap<WayId, Vec<NodeId>> {
    let now = Instant::now();

    println!("parsing ways...");
    let _rewind_result = pbf.rewind();
    let way_to_nodes: HashMap<WayId, Vec<NodeId>> = pbf
        .par_iter()
        .map(Result::unwrap)
        .filter(|obj| obj.is_way())
        .filter(|obj| way_ids.contains(&obj.way().unwrap().id))
        .map(|obj| obj.way().unwrap().clone())
        .map(|way| (way.id, way.nodes))
        .collect();

    println!("parsing ways finished! {}s", now.elapsed().as_secs());
    way_to_nodes
}

fn find_nodes_for_node_ids(pbf: &mut OsmPbfReaderFile, node_ids: HashSet<NodeId>) -> HashMap<NodeId, Node> {
    let now = Instant::now();

    println!("parsing nodes...");
    let _rewind_result = pbf.rewind();
    let node_id_to_node: HashMap<NodeId, Node> = pbf
        .par_iter()
        .map(Result::unwrap)
        .filter(|obj| obj.is_node())
        .filter(|obj| node_ids.contains(&obj.node().unwrap().id))
        .map(|obj| obj.node().unwrap().clone())
        .map(|node| (node.id, node))
        .collect();

    println!("parsing nodes finished! {}s", now.elapsed().as_secs());
    node_id_to_node
}

// ////////////////////////////////////
// ////////////////////////////////////
// UNIT TESTS
// ////////////////////////////////////
// ////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use osmpbfreader::Tags;
    use std::iter::FromIterator;

    use smartstring::alias::String;

    #[test]
    fn test_empty_relation_has_not_proper_admin_level() {
        let relation = create_relation(vec![]);
        assert_eq!(has_proper_admin_level(&relation, &1, &8), false);
    }

    #[test]
    fn test_admin_level_too_high_is_not_valid() {
        let max_admin_level = 8;
        let relation = create_relation(vec![(
            String::from("admin_level"),
            String::from((max_admin_level + 1).to_string()),
        )]);
        assert_eq!(has_proper_admin_level(&relation, &1, &max_admin_level), false);
    }

    #[test]
    fn test_admin_level_is_max_level_is_valid() {
        let max_admin_level = 8;
        let relation = create_relation(vec![(
            String::from("admin_level"),
            String::from((max_admin_level).to_string()),
        )]);
        assert_eq!(has_proper_admin_level(&relation, &1, &max_admin_level), true);
    }

    #[test]
    fn test_min_admin_level_filters_out() {
        let min_admin_level = 1;
        let relation = create_relation(vec![(String::from("admin_level"), String::from((0).to_string()))]);
        assert_eq!(has_proper_admin_level(&relation, &min_admin_level, &8), false);
    }

    #[test]
    fn test_min_equal_max_let_only_exact_level_through() {
        let min_admin_level = 3;
        let max_admin_level = min_admin_level;

        let relation_too_little = create_relation(vec![(
            String::from("admin_level"),
            String::from((min_admin_level - 1).to_string()),
        )]);
        let relation_exact = create_relation(vec![(
            String::from("admin_level"),
            String::from((min_admin_level).to_string()),
        )]);
        let relation_too_big = create_relation(vec![(
            String::from("admin_level"),
            String::from((min_admin_level + 1).to_string()),
        )]);

        assert_eq!(
            has_proper_admin_level(&relation_too_little, &min_admin_level, &max_admin_level),
            false
        );
        assert_eq!(
            has_proper_admin_level(&relation_exact, &min_admin_level, &max_admin_level),
            true
        );
        assert_eq!(
            has_proper_admin_level(&relation_too_big, &min_admin_level, &max_admin_level),
            false
        );
    }

    fn create_relation(tags_pairs: Vec<(String, String)>) -> Relation {
        Relation {
            id: RelationId(123),
            tags: Tags::from_iter(tags_pairs),
            refs: Vec::new(),
        }
    }
}

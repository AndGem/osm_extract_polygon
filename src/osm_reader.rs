use osmpbfreader::{Node, NodeId, OsmPbfReader, Relation, RelationId, WayId};

use std::collections::{HashMap, HashSet};
use std::i8::MAX;
use std::time::Instant;

use crate::utils::hashmap_values_to_set;

type OsmPbfReaderFile = osmpbfreader::OsmPbfReader<std::fs::File>;

#[derive(Clone)]
pub struct RelationNodes {
    pub relation: Relation,
    pub nodes: Vec<Vec<Node>>,
}

pub fn read_osm(filename: &str) -> Vec<RelationNodes> {
    let file_reference = std::fs::File::open(&std::path::Path::new(filename)).unwrap();
    read_ways_and_relation(file_reference)
}

fn read_ways_and_relation(file_reference: std::fs::File) -> Vec<RelationNodes> {
    let mut pbf: OsmPbfReaderFile = OsmPbfReader::new(file_reference);

    let relation_id_to_relation = find_admin_boundary_relations(&mut pbf);

    let relation_id_to_ways: HashMap<RelationId, Vec<WayId>> = find_ways_for_relation_ids(&relation_id_to_relation);
    let way_id_to_node_ids: HashMap<WayId, Vec<NodeId>> =
        find_nodes_for_way_ids(&mut pbf, hashmap_values_to_set(&relation_id_to_ways));
    let node_id_to_node: HashMap<NodeId, Node> =
        find_nodes_for_node_ids(&mut pbf, hashmap_values_to_set(&way_id_to_node_ids));

    let relation_to_nodes: Vec<RelationNodes> = relation_id_to_ways
        .iter()
        .map(|(r_id, way_ids)| (*r_id, replace_way_id_with_node_ids(&way_ids, &way_id_to_node_ids)))
        .map(|(r_id, node_ids)| (r_id, replace_node_id_with_node(node_ids, &node_id_to_node)))
        .map(|(r_id, node_ids)| (relation_id_to_relation.get(&r_id).unwrap().clone(), node_ids))
        .map(|(relation, nodes)| RelationNodes { relation, nodes })
        .collect();

    relation_to_nodes
}

fn has_proper_admin_level(relation: &Relation) -> bool {
    let admin_level: i8 = relation
        .tags
        .get("admin_level")
        .and_then(|v| v.parse::<i8>().ok())
        .unwrap_or(MAX);

    admin_level <= 8
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
        .filter_map(|way_id| way_id_to_node_ids.get(&way_id))
        .cloned()
        .collect()
}

fn replace_node_id_with_node(v_node_ids: Vec<Vec<NodeId>>, node_id_to_node: &HashMap<NodeId, Node>) -> Vec<Vec<Node>> {
    v_node_ids
        .iter()
        .map(|node_ids| {
            node_ids
                .iter()
                .filter_map(|node_id| node_id_to_node.get(&node_id))
                .cloned()
                .collect()
        })
        .collect()
}

fn find_admin_boundary_relations(pbf: &mut OsmPbfReaderFile) -> HashMap<RelationId, Relation> {
    let now = Instant::now();
    println!("parsing relations...");

    let relation_id_to_relation: HashMap<RelationId, Relation> = pbf
        .par_iter()
        .map(Result::unwrap)
        .filter(|obj| obj.is_relation())
        .map(|obj| obj.relation().unwrap().clone())
        .filter(|relation| relation.tags.contains("boundary", "administrative"))
        .filter(|relation| has_proper_admin_level(relation))
        .map(|relation| (relation.id, relation))
        .collect();

    println!("parsing relations finished! {}s", now.elapsed().as_secs());
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
        .map(|obj| obj.way().unwrap().clone())
        .filter(|way| way_ids.contains(&way.id))
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
        .map(|obj| obj.node().unwrap().clone())
        .filter(|node| node_ids.contains(&node.id))
        .map(|node| (node.id, node))
        .collect();

    println!("parsing nodes finished! {}s", now.elapsed().as_secs());
    node_id_to_node
}

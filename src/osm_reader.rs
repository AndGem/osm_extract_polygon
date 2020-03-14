use osmpbfreader::{Node, NodeId, OsmPbfReader, Relation, RelationId, WayId};

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use std::i8::MAX;

type OsmPbfReaderFile = osmpbfreader::OsmPbfReader<std::fs::File>;

#[derive(Clone)]
pub struct RelationNodes {
    pub relation: Relation,
    pub nodes: Vec<Vec<Node>>,
}

use std::fmt;
impl fmt::Debug for RelationNodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RelationNodes {{ data: {:?}, points: {:?} }}",
            self.relation, 0
        )
    }
}

pub fn read_osm(filename: &str) -> Vec<RelationNodes> {
    let file_reference = std::fs::File::open(&std::path::Path::new(filename)).unwrap();
    read_ways_and_relation(file_reference)
}

fn read_ways_and_relation(file_reference: std::fs::File) -> Vec<RelationNodes> {
    let mut pbf: OsmPbfReaderFile = OsmPbfReader::new(file_reference);

    let (relation_id_to_relation, relation_id_to_ways) = find_admin_boundary_relations(&mut pbf);

    let way_ids_for_relations: HashSet<WayId> = relation_id_to_ways
        .iter()
        .flat_map(|(_, v)| v.clone())
        .collect();
    let way_id_to_nodes: HashMap<WayId, Vec<NodeId>> =
        find_nodes_for_way_ids(&mut pbf, way_ids_for_relations);

    let node_ids_for_ways: HashSet<NodeId> = way_id_to_nodes
        .iter()
        .flat_map(|(_, v)| v.clone())
        .collect();
    let node_id_to_node: HashMap<NodeId, Node> =
        find_nodes_for_node_ids(&mut pbf, node_ids_for_ways);

    //TODO: make this nicer as well!
    let mut relation_to_nodes: HashMap<RelationId, Vec<Vec<Node>>> = HashMap::new();
    for (relation_id, way_ids) in relation_id_to_ways {
        for way_id in way_ids {
            let opt_node_ids = way_id_to_nodes.get(&way_id);
            if opt_node_ids.is_none() {
                continue;
            }
            let node_ids: Vec<NodeId> = opt_node_ids.unwrap().clone();

            let nodes: Vec<Node> = node_ids
                .iter()
                .filter_map(|x| node_id_to_node.get(&x))
                .cloned()
                .collect();

            relation_to_nodes
                .entry(relation_id)
                .or_insert_with(Vec::new)
                .push(nodes);
        }
    }

    //prepare output
    let output: Vec<RelationNodes> = relation_to_nodes
        .iter()
        .map(|(r_id, nodes)| RelationNodes {
            relation: relation_id_to_relation.get(&r_id).unwrap().clone(),
            nodes: nodes.to_vec(),
        })
        .collect();

    output
}

fn has_proper_admin_level(relation: &Relation) -> bool {
    let admin_level: i8 = relation
        .tags
        .get("admin_level")
        .and_then(|v| v.parse::<i8>().ok())
        .unwrap_or(MAX);

    admin_level <= 8
}

fn get_ways(relation: &Relation) -> Vec<WayId> {
    relation
        .refs
        .iter()
        .filter_map(|r| r.member.way())
        .collect()
}

fn find_admin_boundary_relations(
    pbf: &mut OsmPbfReaderFile,
) -> (
    HashMap<RelationId, Relation>,
    HashMap<RelationId, Vec<WayId>>,
) {
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

    println!("other stuff!");

    let relation_to_ways: HashMap<RelationId, Vec<WayId>> = relation_id_to_relation
        .iter()
        .map(|(relation_id, relation)| (*relation_id, get_ways(relation)))
        .collect();

    println!("parsing relations finished! {}s", now.elapsed().as_secs());

    (relation_id_to_relation, relation_to_ways)
}

fn find_nodes_for_way_ids(
    pbf: &mut OsmPbfReaderFile,
    way_ids: HashSet<WayId>,
) -> HashMap<WayId, Vec<NodeId>> {
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

fn find_nodes_for_node_ids(
    pbf: &mut OsmPbfReaderFile,
    node_ids: HashSet<NodeId>,
) -> HashMap<NodeId, Node> {
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

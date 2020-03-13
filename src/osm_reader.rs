use osmpbfreader::{Node, NodeId, OsmId, OsmObj, OsmPbfReader, Relation, RelationId, WayId};

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

    let mut relation_id_to_relation: HashMap<RelationId, Relation> = HashMap::new();
    let mut relation_to_nodes: HashMap<RelationId, Vec<Vec<Node>>> = HashMap::new();

    let mut relation_to_ways: HashMap<RelationId, Vec<WayId>> = HashMap::new();

    let mut now = Instant::now();

    println!("parsing relations...");
    for obj in pbf.par_iter().map(Result::unwrap) {
        if let OsmObj::Relation(relation) = obj {
            //TOOD: do it similarly https://github.com/TeXitoi/osmpbfreader-rs/blob/3be099f8b35b0135c35e7d4050aa92807d4be243/src/reader.rs
            if !relation.tags.contains("boundary", "administrative")
                || !relation.tags.contains_key("admin_level")
            {
                continue;
            }

            let admin_level_parse = relation
                .tags
                .get("admin_level")
                .unwrap()
                .parse::<i8>()
                .unwrap_or(MAX);

            if admin_level_parse > 8 {
                continue;
            }

            //TODO: this can be made nicer!
            for entry in &relation.refs {
                if !(entry.member.is_way()) {
                    continue;
                }

                let way_id = entry.member.way().unwrap();

                relation_to_ways
                    .entry(relation.id)
                    .or_insert_with(Vec::new)
                    .push(way_id);
            }

            relation_id_to_relation.insert(relation.id, relation);
        }
    }

    println!("parsing relations finished! {}s", now.elapsed().as_secs());
    now = Instant::now();

    let way_ids: HashSet<WayId> = relation_to_ways
        .iter()
        .flat_map(|(_, v)| v.clone())
        .collect();

    let way_to_nodes: HashMap<WayId, Vec<NodeId>> = map_way_to_nodes(&mut pbf, way_ids);

    let node_ids: HashSet<NodeId> = way_to_nodes.iter().flat_map(|(_, v)| v.clone()).collect();
    let node_id_to_node = map_node_ids_to_nodes(&mut pbf, node_ids);

    //TODO: make this nicer as well!
    for (relation_id, way_ids) in relation_to_ways {
        for way_id in way_ids {
            let opt_node_ids = way_to_nodes.get(&way_id);
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
    println!("parsing nodes finished! {}s", now.elapsed().as_secs());

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

fn map_way_to_nodes(pbf: &mut OsmPbfReaderFile, way_ids: HashSet<WayId>) -> HashMap<WayId, Vec<NodeId>> {
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

fn map_node_ids_to_nodes(pbf: &mut OsmPbfReaderFile, node_ids: HashSet<NodeId>) -> HashMap<NodeId, Node> {
    let now = Instant::now();

    println!("parsing nodes...");
    let _rewind_result = pbf.rewind();
    let node_id_to_node: HashMap<NodeId, Node> = pbf
        .par_iter()
        .map(Result::unwrap)
        .filter(|obj| obj.is_node())
        .map(|obj| obj.node().unwrap().clone())
        .filter(|node| node_ids.contains(&node.id))
        .map(|node| (node.id, node.clone()))
        .collect();

    println!("parsing nodes finished! {}s", now.elapsed().as_secs());
    node_id_to_node
}

// Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;

type TestDag<'a> = Merkle<
    BTreeMap<Vec<u8>, Node<std::collections::hash_map::DefaultHasher>>,
    std::collections::hash_map::DefaultHasher,
>;

#[test]
fn test_root_pointer_hygiene() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quax_node_id = dag.add_node("quax", BTreeSet::new()).unwrap();
    assert_eq!(
        quax_node_id,
        *dag.get_node_by_id(&quax_node_id).unwrap().unwrap().id()
    );
    assert!(dag.get_roots().contains(&quax_node_id));
    let mut dep_set = BTreeSet::new();
    dep_set.insert(quax_node_id.clone());
    let quux_node_id = dag.add_node("quux", dep_set).unwrap();
    assert!(!dag.get_roots().contains(&quax_node_id));
    assert!(dag.get_roots().contains(&quux_node_id));
    assert_eq!(
        quux_node_id,
        *dag.get_node_by_id(&quux_node_id).unwrap().unwrap().id()
    );
}

#[test]
fn test_insert_no_such_dependents_error() {
    let missing_dependent =
        Node::<DefaultHasher>::new("missing".as_bytes().to_vec(), BTreeSet::new());
    let mut dag = TestDag::new(BTreeMap::new());
    let mut dep_set = BTreeSet::new();
    dep_set.insert(missing_dependent.id().to_vec());
    assert!(dag.add_node("foo", dep_set).is_err());
    assert!(dag.get_roots().is_empty());
    assert!(dag.get_nodes().is_empty());
}

#[test]
fn test_adding_nodes_is_idempotent() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quax_node_id = dag.add_node("quax", BTreeSet::new()).unwrap();
    assert_eq!(
        quax_node_id,
        *dag.get_node_by_id(&quax_node_id).unwrap().unwrap().id()
    );
    assert!(dag.get_roots().contains(&quax_node_id));
    let root_size = dag.get_roots().len();
    let nodes_size = dag.get_nodes().len();
    dag.add_node("quax", BTreeSet::new()).unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());
}

#[test]
fn test_adding_nodes_is_idempotent_regardless_of_dep_order() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag.add_node("qualm", BTreeSet::new()).unwrap();
    let quell_node_id = dag.add_node("quell", BTreeSet::new()).unwrap();
    let dep_ids = BTreeSet::from([
        quake_node_id.clone(),
        qualm_node_id.clone(),
        quell_node_id.clone(),
    ]);
    dag.add_node("foo", dep_ids).unwrap();
    let root_size = dag.get_roots().len();
    let nodes_size = dag.get_nodes().len();

    let dep_ids = BTreeSet::from([
        quell_node_id.clone(),
        quake_node_id.clone(),
        qualm_node_id.clone(),
    ]);
    dag.add_node("foo", dep_ids).unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());

    let dep_ids = BTreeSet::from([
        qualm_node_id.clone(),
        quell_node_id.clone(),
        quake_node_id.clone(),
    ]);
    dag.add_node("foo", dep_ids).unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());
}

#[test]
fn test_node_comparison_equivalent() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    assert_eq!(
        dag.compare(&quake_node_id, &quake_node_id).unwrap(),
        NodeCompare::Equivalent
    );
}

#[test]
fn test_node_comparison_before() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag
        .add_node("qualm", BTreeSet::from([quake_node_id.clone()]))
        .unwrap();
    let quell_node_id = dag
        .add_node("quell", BTreeSet::from([qualm_node_id.clone()]))
        .unwrap();
    assert_eq!(
        dag.compare(&quake_node_id, &qualm_node_id).unwrap(),
        NodeCompare::Before
    );
    assert_eq!(
        dag.compare(&quake_node_id, &quell_node_id).unwrap(),
        NodeCompare::Before
    );
}

#[test]
fn test_node_comparison_after() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag
        .add_node("qualm", BTreeSet::from([quake_node_id.clone()]))
        .unwrap();
    let quell_node_id = dag
        .add_node("quell", BTreeSet::from([qualm_node_id.clone()]))
        .unwrap();
    assert_eq!(
        dag.compare(&qualm_node_id, &quake_node_id).unwrap(),
        NodeCompare::After
    );
    assert_eq!(
        dag.compare(&quell_node_id, &quake_node_id).unwrap(),
        NodeCompare::After
    );
}

#[test]
fn test_node_comparison_no_shared_graph() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag.add_node("qualm", BTreeSet::new()).unwrap();
    let quell_node_id = dag.add_node("quell", BTreeSet::new()).unwrap();
    assert_eq!(
        dag.compare(&qualm_node_id, &quake_node_id).unwrap(),
        NodeCompare::Uncomparable
    );
    assert_eq!(
        dag.compare(&quell_node_id, &quake_node_id).unwrap(),
        NodeCompare::Uncomparable
    );
    assert_eq!(
        dag.compare(&quell_node_id, &qualm_node_id).unwrap(),
        NodeCompare::Uncomparable
    );
}

#[cfg(feature = "cbor")]
mod cbor_serialization_tests {
    use super::TestDag;
    use crate::prelude::*;
    use ciborium::{de::from_reader, ser::into_writer};
    use std::collections::BTreeMap;
    use std::collections::{hash_map::DefaultHasher, BTreeSet};

    #[test]
    fn test_node_deserializaton() {
        let mut dag = TestDag::new(BTreeMap::new());
        let simple_node_id = dag.add_node("simple", BTreeSet::new()).unwrap();
        let mut dep_set = BTreeSet::new();
        dep_set.insert(simple_node_id.clone());
        let root_node_id = dag.add_node("root", dep_set).unwrap();

        let simple_node_to_serialize = dag
            .get_node_by_id(simple_node_id.as_slice())
            .unwrap()
            .unwrap();
        let root_node_to_serialize = dag
            .get_node_by_id(root_node_id.as_slice())
            .unwrap()
            .unwrap();

        let mut simple_node_vec: Vec<u8> = Vec::new();
        let mut root_node_vec: Vec<u8> = Vec::new();
        into_writer(&simple_node_to_serialize, &mut simple_node_vec).unwrap();
        into_writer(&root_node_to_serialize, &mut root_node_vec).unwrap();

        let simple_node_de: Node<DefaultHasher> = from_reader(simple_node_vec.as_slice()).unwrap();
        let root_node_de: Node<DefaultHasher> = from_reader(root_node_vec.as_slice()).unwrap();
        assert_eq!(simple_node_to_serialize.id(), simple_node_de.id());
        assert_eq!(simple_node_to_serialize.item_id(), simple_node_de.item_id());
        assert_eq!(simple_node_to_serialize.item(), simple_node_de.item());
        assert_eq!(
            simple_node_to_serialize.dependency_ids(),
            simple_node_de.dependency_ids()
        );
        assert_eq!(root_node_to_serialize.id(), root_node_de.id());
        assert_eq!(root_node_to_serialize.item_id(), root_node_de.item_id());
        assert_eq!(root_node_to_serialize.item(), root_node_de.item());
        assert_eq!(
            root_node_to_serialize.dependency_ids(),
            root_node_de.dependency_ids()
        );
    }
}

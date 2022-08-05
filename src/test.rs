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
    BTreeMap<[u8; 8], Node<&'a str, std::collections::hash_map::DefaultHasher, 8>>,
    &'a str,
    std::collections::hash_map::DefaultHasher,
    8,
>;

#[test]
fn test_root_pointer_hygiene() {
    let mut dag = TestDag::new();
    let quax_node_id = dag.add_node("quax", BTreeSet::new()).unwrap();
    assert_eq!(
        quax_node_id,
        *dag.get_node_by_id(&quax_node_id).unwrap().id()
    );
    assert!(dag.get_roots().contains(&quax_node_id));
    let mut dep_set = BTreeSet::new();
    dep_set.insert(quax_node_id);
    let quux_node_id = dag.add_node("quux", dep_set).unwrap();
    assert!(!dag.get_roots().contains(&quax_node_id));
    assert!(dag.get_roots().contains(&quux_node_id));
    assert_eq!(
        quux_node_id,
        *dag.get_node_by_id(&quux_node_id).unwrap().id()
    );
}

#[test]
fn test_insert_no_such_dependents_error() {
    let missing_dependent = Node::<&str, DefaultHasher, 8>::new("missing", BTreeSet::new());
    let mut dag = TestDag::new();
    let mut dep_set = BTreeSet::new();
    dep_set.insert(*missing_dependent.id());
    assert!(dag.add_node("foo", dep_set).is_err());
    assert!(dag.get_roots().is_empty());
    assert!(dag.get_nodes().is_empty());
}

#[test]
fn test_adding_nodes_is_idempotent() {
    let mut dag = TestDag::new();
    let quax_node_id = dag.add_node("quax", BTreeSet::new()).unwrap();
    assert_eq!(
        quax_node_id,
        *dag.get_node_by_id(&quax_node_id).unwrap().id()
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
    let mut dag = TestDag::new();
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag.add_node("qualm", BTreeSet::new()).unwrap();
    let quell_node_id = dag.add_node("quell", BTreeSet::new()).unwrap();
    let dep_ids = BTreeSet::from([quake_node_id, qualm_node_id, quell_node_id]);
    dag.add_node("foo", dep_ids).unwrap();
    let root_size = dag.get_roots().len();
    let nodes_size = dag.get_nodes().len();

    let dep_ids = BTreeSet::from([quell_node_id, quake_node_id, qualm_node_id]);
    dag.add_node("foo", dep_ids).unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());

    let dep_ids = BTreeSet::from([qualm_node_id, quell_node_id, quake_node_id]);
    dag.add_node("foo", dep_ids).unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());
}

#[test]
fn test_node_comparison_equivalent() {
    let mut dag = TestDag::new();
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    assert_eq!(
        dag.compare(&quake_node_id, &quake_node_id),
        NodeCompare::Equivalent
    );
}

#[test]
fn test_node_comparison_before() {
    let mut dag = TestDag::new();
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag
        .add_node("qualm", BTreeSet::from([quake_node_id.clone()]))
        .unwrap();
    let quell_node_id = dag
        .add_node("quell", BTreeSet::from([qualm_node_id.clone()]))
        .unwrap();
    assert_eq!(
        dag.compare(&quake_node_id, &qualm_node_id),
        NodeCompare::Before
    );
    assert_eq!(
        dag.compare(&quake_node_id, &quell_node_id),
        NodeCompare::Before
    );
}

#[test]
fn test_node_comparison_after() {
    let mut dag = TestDag::new();
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag
        .add_node("qualm", BTreeSet::from([quake_node_id.clone()]))
        .unwrap();
    let quell_node_id = dag
        .add_node("quell", BTreeSet::from([qualm_node_id.clone()]))
        .unwrap();
    assert_eq!(
        dag.compare(&qualm_node_id, &quake_node_id),
        NodeCompare::After
    );
    assert_eq!(
        dag.compare(&quell_node_id, &quake_node_id),
        NodeCompare::After
    );
}

#[test]
fn test_node_comparison_no_shared_graph() {
    let mut dag = TestDag::new();
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).unwrap();
    let qualm_node_id = dag.add_node("qualm", BTreeSet::new()).unwrap();
    let quell_node_id = dag.add_node("quell", BTreeSet::new()).unwrap();
    assert_eq!(
        dag.compare(&qualm_node_id, &quake_node_id),
        NodeCompare::Uncomparable
    );
    assert_eq!(
        dag.compare(&quell_node_id, &quake_node_id),
        NodeCompare::Uncomparable
    );
    assert_eq!(
        dag.compare(&quell_node_id, &qualm_node_id),
        NodeCompare::Uncomparable
    );
}

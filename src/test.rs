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

use super::*;

#[test]
fn test_root_pointer_hygiene() {
    let mut dag = DAG::<&str, DefaultHasher, 8>::new();
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
    let mut dag = DAG::<&str, DefaultHasher, 8>::new();
    let mut dep_set = BTreeSet::new();
    dep_set.insert(*missing_dependent.id());
    assert!(dag.add_node("foo", dep_set).is_err());
    assert!(dag.get_roots().is_empty());
    assert!(dag.get_nodes().is_empty());
}

#[test]
fn test_adding_nodes_is_idempotent() {
    let mut dag = DAG::<&str, DefaultHasher, 8>::new();
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

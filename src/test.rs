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
use async_std;
use crate::prelude::*;

type TestDag<'a> = Merkle<
    BTreeMap<Vec<u8>, Node<std::collections::hash_map::DefaultHasher>>,
    std::collections::hash_map::DefaultHasher,
>;

#[async_std::test]
async fn test_root_pointer_hygiene() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quax_node_id = dag.add_node("quax", BTreeSet::new()).await.unwrap();
    assert_eq!(
        quax_node_id,
        *dag.get_node_by_id(&quax_node_id).await.unwrap().unwrap().id()
    );
    assert!(dag.get_roots().contains(&quax_node_id));
    let mut dep_set = BTreeSet::new();
    dep_set.insert(quax_node_id.clone());
    let quux_node_id = dag.add_node("quux", dep_set).await.unwrap();
    assert!(!dag.get_roots().contains(&quax_node_id));
    assert!(dag.get_roots().contains(&quux_node_id));
    assert_eq!(
        quux_node_id,
        *dag.get_node_by_id(&quux_node_id).await.unwrap().unwrap().id()
    );
}

#[async_std::test]
async fn test_insert_no_such_dependents_error() {
    let missing_dependent =
        Node::<DefaultHasher>::new("missing".as_bytes().to_vec(), BTreeSet::new());
    let mut dag = TestDag::new(BTreeMap::new());
    let mut dep_set = BTreeSet::new();
    dep_set.insert(missing_dependent.id().to_vec());
    assert!(dag.add_node("foo", dep_set).await.is_err());
    assert!(dag.get_roots().is_empty());
    assert!(dag.get_nodes().is_empty());
}

#[async_std::test]
async fn test_adding_nodes_is_idempotent() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quax_node_id = dag.add_node("quax", BTreeSet::new()).await.unwrap();
    assert_eq!(
        quax_node_id,
        *dag.get_node_by_id(&quax_node_id).await.unwrap().unwrap().id()
    );
    assert!(dag.get_roots().contains(&quax_node_id));
    let root_size = dag.get_roots().len();
    let nodes_size = dag.get_nodes().len();
    dag.add_node("quax", BTreeSet::new()).await.unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());
}

#[async_std::test]
async fn test_adding_nodes_is_idempotent_regardless_of_dep_order() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).await.unwrap();
    let qualm_node_id = dag.add_node("qualm", BTreeSet::new()).await.unwrap();
    let quell_node_id = dag.add_node("quell", BTreeSet::new()).await.unwrap();
    let dep_ids = BTreeSet::from([
        quake_node_id.clone(),
        qualm_node_id.clone(),
        quell_node_id.clone(),
    ]);
    dag.add_node("foo", dep_ids).await.unwrap();
    let root_size = dag.get_roots().len();
    let nodes_size = dag.get_nodes().len();

    let dep_ids = BTreeSet::from([
        quell_node_id.clone(),
        quake_node_id.clone(),
        qualm_node_id.clone(),
    ]);
    dag.add_node("foo", dep_ids).await.unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());

    let dep_ids = BTreeSet::from([
        qualm_node_id.clone(),
        quell_node_id.clone(),
        quake_node_id.clone(),
    ]);
    dag.add_node("foo", dep_ids).await.unwrap();
    assert_eq!(root_size, dag.get_roots().len());
    assert_eq!(nodes_size, dag.get_nodes().len());
}

#[async_std::test]
async fn test_node_comparison_equivalent() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).await.unwrap();
    assert_eq!(
        dag.compare(&quake_node_id, &quake_node_id).await.unwrap(),
        NodeCompare::Equivalent
    );
}

#[async_std::test]
async fn test_node_comparison_before() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).await.unwrap();
    let qualm_node_id = dag
        .add_node("qualm", BTreeSet::from([quake_node_id.clone()])).await
        .unwrap();
    let quell_node_id = dag
        .add_node("quell", BTreeSet::from([qualm_node_id.clone()])).await
        .unwrap();
    assert_eq!(
        dag.compare(&quake_node_id, &qualm_node_id).await.unwrap(),
        NodeCompare::Before
    );
    assert_eq!(
        dag.compare(&quake_node_id, &quell_node_id).await.unwrap(),
        NodeCompare::Before
    );
}

#[async_std::test]
async fn test_node_comparison_after() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).await.unwrap();
    let qualm_node_id = dag
        .add_node("qualm", BTreeSet::from([quake_node_id.clone()])).await
        .unwrap();
    let quell_node_id = dag
        .add_node("quell", BTreeSet::from([qualm_node_id.clone()])).await
        .unwrap();
    assert_eq!(
        dag.compare(&qualm_node_id, &quake_node_id).await.unwrap(),
        NodeCompare::After
    );
    assert_eq!(
        dag.compare(&quell_node_id, &quake_node_id).await.unwrap(),
        NodeCompare::After
    );
}

#[async_std::test]
async fn test_node_comparison_no_shared_graph() {
    let mut dag = TestDag::new(BTreeMap::new());
    let quake_node_id = dag.add_node("quake", BTreeSet::new()).await.unwrap();
    let qualm_node_id = dag.add_node("qualm", BTreeSet::new()).await.unwrap();
    let quell_node_id = dag.add_node("quell", BTreeSet::new()).await.unwrap();
    assert_eq!(
        dag.compare(&qualm_node_id, &quake_node_id).await.unwrap(),
        NodeCompare::Uncomparable
    );
    assert_eq!(
        dag.compare(&quell_node_id, &quake_node_id).await.unwrap(),
        NodeCompare::Uncomparable
    );
    assert_eq!(
        dag.compare(&quell_node_id, &qualm_node_id).await.unwrap(),
        NodeCompare::Uncomparable
    );
}

#[async_std::test]
async fn test_find_next_missing_nodes_disjoint_graphs_no_deps() {
    let mut dag1 = TestDag::new(BTreeMap::new());
    let mut dag2 = TestDag::new(BTreeMap::new());
    let quake_node_id = dag1.add_node("quake", BTreeSet::new()).await.unwrap();
    let qualm_node_id = dag1.add_node("qualm", BTreeSet::new()).await.unwrap();
    dag2.add_node("quell", BTreeSet::new()).await.unwrap();
    let missing_nodes = dag1
        .find_next_non_descendant_nodes(dag2.get_roots()).await
        .unwrap();
    assert_eq!(missing_nodes.len(), 2);
    let mut found_quake = false;
    let mut found_qualm = false;
    for node in missing_nodes {
        if node.id().to_owned() == quake_node_id {
            found_quake = true;
        }
        if node.id().to_owned() == qualm_node_id {
            found_qualm = true;
        }
    }
    assert!(found_quake);
    assert!(found_qualm);
}

#[async_std::test]
async fn test_find_next_missing_nodes_sub_graphs_one_degree_off() {
    let mut dag1 = TestDag::new(BTreeMap::new());
    let mut dag2 = TestDag::new(BTreeMap::new());
    dag1.add_node("quake", BTreeSet::new()).await.unwrap();
    let quake_node_id = dag2.add_node("quake", BTreeSet::new()).await.unwrap();

    let mut deps = BTreeSet::new();
    deps.insert(quake_node_id);
    let qualm_node_id = dag1.add_node("qualm", deps).await.unwrap();

    let missing_nodes = dag1
        .find_next_non_descendant_nodes(dag2.get_roots()).await
        .unwrap();
    assert_eq!(missing_nodes.len(), 1);
    let mut found_qualm = false;
    for node in missing_nodes {
        if node.id().to_owned() == qualm_node_id {
            found_qualm = true;
        }
    }
    assert!(found_qualm);
}

#[async_std::test]
async fn test_find_next_missing_nodes_sub_graphs_two_degree_off() {
    let mut dag1 = TestDag::new(BTreeMap::new());
    let mut dag2 = TestDag::new(BTreeMap::new());
    dag1.add_node("quake", BTreeSet::new()).await.unwrap();
    let quake_node_id = dag2.add_node("quake", BTreeSet::new()).await.unwrap();

    let mut deps = BTreeSet::new();
    deps.insert(quake_node_id.clone());
    let qualm_node_id = dag1.add_node("qualm", deps).await.unwrap();

    deps = BTreeSet::new();
    deps.insert(quake_node_id.clone());
    deps.insert(qualm_node_id.clone());
    let quell_node_id = dag1.add_node("quell", deps).await.unwrap();

    let missing_nodes = dag1
        .find_next_non_descendant_nodes(dag2.get_roots()).await
        .unwrap();
    assert_eq!(missing_nodes.len(), 2);
    let mut found_qualm = false;
    let mut found_quell = false;
    for node in missing_nodes {
        if node.id().to_owned() == qualm_node_id {
            found_qualm = true;
        }
        if node.id().to_owned() == quell_node_id {
            found_quell = true;
        }
    }
    assert!(found_qualm);
    assert!(found_quell);
}

#[cfg(feature = "cbor")]
mod cbor_serialization_tests {
    use super::TestDag;
    use crate::prelude::*;
    use ciborium::{de::from_reader, ser::into_writer};
    use std::collections::BTreeMap;
    use std::collections::{hash_map::DefaultHasher, BTreeSet};

    #[async_std::test]
    async fn test_node_deserializaton() {
        let mut dag = TestDag::new(BTreeMap::new());
        let simple_node_id = dag.add_node("simple", BTreeSet::new()).await.unwrap();
        let mut dep_set = BTreeSet::new();
        dep_set.insert(simple_node_id.clone());
        let root_node_id = dag.add_node("root", dep_set).await.unwrap();

        let simple_node_to_serialize = dag
            .get_node_by_id(simple_node_id.as_slice()).await
            .unwrap()
            .unwrap();
        let root_node_to_serialize = dag
            .get_node_by_id(root_node_id.as_slice()).await
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

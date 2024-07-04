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
use async_std::task::block_on;
use std::collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet};

use proptest::prelude::*;

use crate::prelude::*;

type TestDag = Merkle<BTreeMap<Vec<u8>, Node<DefaultHasher>>, DefaultHasher>;

fn simple_edge_strategy(
    nodes_count: usize,
) -> impl Strategy<Value = (Vec<String>, BTreeSet<usize>)> {
    prop::collection::vec(".*", 4..nodes_count).prop_flat_map(|payloads| {
        let nodes_len = payloads.len();
        (
            // our total list of nodes.
            Just(payloads),
            // our list of roots.
            prop::collection::btree_set(1..nodes_len, 1..(nodes_len / 2)),
        )
    })
}

fn complex_dag_strategy(
    nodes_count: usize,
    depth: usize,
    branch: usize,
) -> impl Strategy<Value = TestDag> {
    prop::collection::vec(".*", depth..nodes_count).prop_flat_map(move |payloads| {
        let nodes_len = payloads.len();
        let mut dag = TestDag::new(BTreeMap::new());
        // partition the payloads into depth pieces
        let mut id_stack: Vec<Vec<u8>> = Vec::new();
        for chunk in payloads.chunks(nodes_len / depth) {
            // loop through the partions adding each partions nodes to the dag.
            let dep_sets: Vec<BTreeSet<Vec<u8>>> = if id_stack.is_empty() {
                vec![BTreeSet::new()]
            } else {
                let mut dep_sets = Vec::new();
                for id_chunk in id_stack.chunks(branch) {
                    let id_set = id_chunk.iter().fold(BTreeSet::new(), |mut acc, item| {
                        acc.insert(item.clone());
                        acc
                    });
                    dep_sets.push(id_set);
                }
                dep_sets
            };
            let dep_set_len = dep_sets.len();
            for (idx, p) in chunk.iter().enumerate() {
                let dep_idx = idx % dep_set_len;
                let dep_set = dep_sets[dep_idx].clone();
                // NOTE(zaphar): We need to block on here.
                block_on( async {
                    id_stack.push(dag.add_node(p.clone(), dep_set).await.unwrap().clone());
                });
            }
        }
        Just(dag)
    })
}

proptest! {
    #[test]
    fn test_dag_add_node_properties((nodes, parent_idxs) in simple_edge_strategy(100)) {
        block_on(async {
            // TODO implement the tests now
            let mut dag = TestDag::new(BTreeMap::new());
            let parent_count = parent_idxs.len();
            let mut dependents = BTreeMap::new();
            let mut node_set = BTreeSet::new();
            for (idx, n) in nodes.iter().cloned().enumerate() {
                if !parent_idxs.contains(&idx) {
                    let node_id = dag.add_node(n.as_bytes(), BTreeSet::new()).await.unwrap();
                    node_set.insert(node_id.clone());
                    let parent = idx % parent_count;
                    if dependents.contains_key(&parent) {
                        dependents.get_mut(&parent).map(|v: &mut BTreeSet<Vec<u8>>| v.insert(node_id));
                    } else {
                        dependents.insert(parent, BTreeSet::from([node_id]));
                    }
                }
            }
            for (pidx, dep_ids) in dependents {
                let node_id = dag.add_node(nodes[pidx].clone(), dep_ids).await.unwrap();
                node_set.insert(node_id.clone());
            }
            assert!(dag.get_roots().len() <= parent_count);
            assert!(dag.get_nodes().len() == node_set.len());
        })
    }
}

proptest! {
    #[test]
    fn test_complex_dag_node_properties(dag in complex_dag_strategy(100, 10, 3)) {
        block_on(async {
            // TODO(jwall): We can assert much more about the Merkle if we get more clever in what we return.
            let nodes = dag.get_nodes();
            assert!(nodes.len() <= 100);

            let roots = dag.get_roots();
            assert!(roots.len() < dag.get_nodes().len());

            for node_id in nodes.keys() {
                let mut is_descendant = false;
                if roots.contains(node_id) {
                    continue;
                }
                for root in roots.iter() {
                    if let NodeCompare::After = dag.compare(root, node_id).await.unwrap() {
                        // success
                        is_descendant = true;
                    }
                }
                assert!(is_descendant);
            }
            // Check that every root node is uncomparable.
            for left_root in roots.iter() {
                for right_root in roots.iter() {
                    if left_root != right_root {
                        assert_eq!(dag.compare(left_root, right_root).await.unwrap(), NodeCompare::Uncomparable);
                    }
                }
            }
        });
    }
}

#[cfg(feature = "cbor")]
proptest! {
    #[test]
    fn test_node_serde_strategy(dag in complex_dag_strategy(100, 10, 3)) {
        use ciborium::{de::from_reader, ser::into_writer};
        block_on(async {
            let nodes = dag.get_nodes();
            for (_, node) in nodes {
                let node = node.clone();
                let mut buf: Vec<u8> = Vec::new();
                into_writer(&node, &mut buf).unwrap();
                let node_de: Node<DefaultHasher> = from_reader(buf.as_slice()).unwrap();
                assert_eq!(node.id(), node_de.id());
                assert_eq!(node.item_id(), node_de.item_id());
                assert_eq!(node.item(), node_de.item());
                assert_eq!(node.dependency_ids(), node_de.dependency_ids());
            }
        });
    }
}

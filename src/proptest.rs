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
use proptest::prelude::*;
use std::collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet};

use crate::{NodeCompare, DAG};

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
) -> impl Strategy<Value = DAG<String, std::collections::hash_map::DefaultHasher, 8>> {
    prop::collection::vec(".*", depth..nodes_count).prop_flat_map(move |payloads| {
        let nodes_len = payloads.len();
        let mut dag = DAG::<String, std::collections::hash_map::DefaultHasher, 8>::new();
        // partition the payloads into depth pieces
        let mut id_stack: Vec<[u8; 8]> = Vec::new();
        for chunk in payloads.chunks(nodes_len / depth) {
            // loop through the partions adding each partions nodes to the dag.
            let dep_sets: Vec<BTreeSet<[u8; 8]>> = if id_stack.is_empty() {
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
                id_stack.push(dag.add_node(p.clone(), dep_set).unwrap().clone());
            }
        }
        Just(dag)
    })
}

proptest! {
    #[test]
    fn test_dag_add_node_properties((nodes, parent_idxs) in simple_edge_strategy(100)) {
        // TODO implement the tests now
        let mut dag = DAG::<String, DefaultHasher, 8>::new();
        let parent_count = parent_idxs.len();
        let mut dependents = BTreeMap::new();
        let mut node_set = BTreeSet::new();
        for (idx, n) in nodes.iter().cloned().enumerate() {
            if !parent_idxs.contains(&idx) {
                let node_id = dag.add_node(n, BTreeSet::new()).unwrap();
                node_set.insert(node_id.clone());
                let parent = idx % parent_count;
                if dependents.contains_key(&parent) {
                    dependents.get_mut(&parent).map(|v: &mut BTreeSet<[u8; 8]>| v.insert(node_id));
                } else {
                    dependents.insert(parent, BTreeSet::from([node_id]));
                }
            }
        }
        for (pidx, dep_ids) in dependents {
            let node_id = dag.add_node(nodes[pidx].clone(), dep_ids).unwrap();
            node_set.insert(node_id.clone());
        }
        assert!(dag.get_roots().len() <= parent_count);
        assert!(dag.get_nodes().len() == node_set.len());
    }
}

proptest! {
    #[test]
    fn test_complex_dag_node_properties(dag in complex_dag_strategy(100, 10, 3)) {
        // TODO(jwall): We can assert much more about the DAG if we get more clever in what we return.
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
                if let NodeCompare::After = dag.compare(root, node_id) {
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
                    assert_eq!(dag.compare(left_root, right_root), NodeCompare::Uncomparable);
                }
            }
        }
    }
}

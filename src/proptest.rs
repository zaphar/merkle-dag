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

use crate::DAG;

fn edge_strategy(nodes_count: usize) -> impl Strategy<Value = (Vec<String>, BTreeSet<usize>)> {
    prop::collection::vec(".*", 4..nodes_count).prop_flat_map(|payloads| {
        let nodes_len = payloads.len();
        // TODO(jwall): Generate valid DAGs
        // select a random set of payloads to be roots.
        // select a random set of non root payloads to be dependencies
        (
            // our total list of nodes.
            Just(payloads),
            // our random list of roots.
            prop::collection::btree_set(1..nodes_len, 1..(nodes_len / 2)),
        )
    })
}

proptest! {
    #[test]
    fn test_dag_add_node_properties((nodes, parent_idxs) in edge_strategy(100)) {
        // TODO implement the tests now
        let mut dag = DAG::<String, DefaultHasher, 8>::new();
        let parent_count = parent_idxs.len();
        let mut dependents = BTreeMap::new();
        for (idx, n) in nodes.iter().cloned().enumerate() {
            if !parent_idxs.contains(&idx) {
                let node_id = dag.add_node(n, BTreeSet::new()).unwrap();
                let parent = idx % parent_count;
                if dependents.contains_key(&parent) {
                    dependents.get_mut(&parent).map(|v: &mut BTreeSet<[u8; 8]>| v.insert(node_id));
                } else {
                    dependents.insert(parent, BTreeSet::from([node_id]));
                }
            }
        }
        for (pidx, dep_ids) in dependents {
            dag.add_node(nodes[pidx].clone(), dep_ids).unwrap();
        }
        assert!(dag.get_roots().len() <= parent_count);
        assert!(dag.get_nodes().len() <= nodes.len());
    }
}

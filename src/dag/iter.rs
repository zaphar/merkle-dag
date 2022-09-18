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
use std::collections::BTreeSet;

use super::Merkle;
use crate::hash::HashWriter;
use crate::node::Node;
use crate::store::{Result, Store};

/// An iterator over the missing nodes in a DAG given a set of root nodes.
pub struct Missing<'dag, S, HW>
where
    S: Store<HW>,
    HW: HashWriter,
{
    dag: &'dag Merkle<S, HW>,
    search_nodes: BTreeSet<Vec<u8>>,
}

impl<'dag, S, HW> Missing<'dag, S, HW>
where
    S: Store<HW>,
    HW: HashWriter,
{
    /// Create an Iterator for the missing nodes given a set of root nodes.
    pub fn new(dag: &'dag Merkle<S, HW>, search_nodes: BTreeSet<Vec<u8>>) -> Self {
        Self { dag, search_nodes }
    }

    /// Returns the next set of missing nodes in the iterator.
    pub fn next(&mut self) -> Result<Option<Vec<Node<HW>>>> {
        let nodes = self
            .dag
            .find_next_non_descendant_nodes(&self.search_nodes)?;
        self.search_nodes = BTreeSet::new();
        for id in nodes.iter().map(|n| n.id().to_vec()) {
            self.search_nodes.insert(id);
        }
        if nodes.len() > 0 {
            Ok(Some(nodes))
        } else {
            Ok(None)
        }
    }
}

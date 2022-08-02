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
use std::collections::{BTreeMap, BTreeSet};

use hash::{ByteEncoder, HashWriter};
use node::Node;

mod hash;
mod node;

pub enum EdgeError {
    NoSuchDependents,
}

/// A Merkle-DAG implementation. This is a modification on the standard Merkle Tree data structure
/// but instead of a tree it is a DAG and as a result can have multiple roots. A merkle-dag specifies
/// a partial ordering on all the nodes and utilizes the api to ensure that this ordering is
/// preserved during construction.
///
/// The merkle dag consists of a set of pointers to the current known roots as well as the total set
/// of nodes in the dag. Node payload items must be of a single type and implement the `ByteEncoder`
/// trait.
///
/// A merkle DAG instance is tied to a specific implementation of the HashWriter interface to ensure
/// that all hash identifiers are of the same hash algorithm.
pub struct DAG<N, HW>
where
    N: ByteEncoder,
    HW: HashWriter,
{
    roots: BTreeSet<Vec<u8>>,
    nodes: BTreeMap<Vec<u8>, Node<N, HW>>,
}

impl<N, HW> DAG<N, HW>
where
    N: ByteEncoder,
    HW: HashWriter,
{
    /// Construct a new empty DAG. The empty DAG is also the default for a DAG.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new payload with a required set of dependency_ids. This method will construct a new node
    /// and add it to the DAG with the given payload item and dependency id set. It is idempotent for any
    /// given set of inputs.
    pub fn add_node(
        &mut self,
        item: N,
        dependency_ids: BTreeSet<Vec<u8>>,
    ) -> Result<(), EdgeError> {
        let node = Node::<N, HW>::new(item, dependency_ids.clone());
        let id = node.id();
        if self.roots.contains(id) {
            // We've already added this node so there is nothing left to do.
            return Ok(());
        }
        for dep_id in dependency_ids.iter() {
            if !self.nodes.contains_key(dep_id) {
                return Err(EdgeError::NoSuchDependents);
            }
        }
        Ok(())
    }

    /// Get a node from the DAG by it's hash identifier if it exists.
    pub fn get_node_by_id(&self, id: &Vec<u8>) -> Option<&Node<N, HW>> {
        self.nodes.get(id)
    }
}

impl<N, HW> Default for DAG<N, HW>
where
    N: ByteEncoder,
    HW: HashWriter,
{
    fn default() -> Self {
        Self {
            roots: BTreeSet::new(),
            nodes: BTreeMap::new(),
        }
    }
}

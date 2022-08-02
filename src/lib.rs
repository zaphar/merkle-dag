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

#[derive(Debug)]
pub enum EdgeError {
    NoSuchDependents,
}
// TODO(jwall): In order to avoid copies it is probably smart to have some concept of
//  a node pool.

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
pub struct DAG<N, HW, const HASH_LEN: usize>
where
    N: ByteEncoder,
    HW: HashWriter<HASH_LEN>,
{
    roots: BTreeSet<[u8; HASH_LEN]>,
    nodes: BTreeMap<[u8; HASH_LEN], Node<N, HW, HASH_LEN>>,
}

impl<N, HW, const HASH_LEN: usize> DAG<N, HW, HASH_LEN>
where
    N: ByteEncoder,
    HW: HashWriter<HASH_LEN>,
{
    /// Construct a new empty DAG. The empty DAG is also the default for a DAG.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new payload with a required set of dependency_ids. This method will construct a new node
    /// and add it to the DAG with the given payload item and dependency id set. It is idempotent for any
    /// given set of inputs.
    pub fn add_node<'a>(
        &'a mut self,
        item: N,
        dependency_ids: BTreeSet<[u8; HASH_LEN]>,
    ) -> Result<[u8; HASH_LEN], EdgeError> {
        let node = Node::<N, HW, HASH_LEN>::new(item, dependency_ids.clone());
        let id = node.id().clone();
        if self.nodes.contains_key(&id) {
            // We've already added this node so there is nothing left to do.
            return Ok(id);
        }
        for dep_id in dependency_ids.iter() {
            if !self.nodes.contains_key(dep_id) {
                return Err(EdgeError::NoSuchDependents);
            }
            // If any of our dependencies is in the roots pointer list then
            // it is time to remove it from there.
            if self.roots.contains(dep_id) {
                self.roots.remove(dep_id);
            }
        }
        self.roots.insert(id.clone());
        self.nodes.insert(id.clone(), node);
        Ok(id)
    }

    /// Get a node from the DAG by it's hash identifier if it exists.
    pub fn get_node_by_id(&self, id: &[u8; HASH_LEN]) -> Option<&Node<N, HW, HASH_LEN>> {
        self.nodes.get(id)
    }

    pub fn get_roots(&self) -> &BTreeSet<[u8; HASH_LEN]> {
        &self.roots
    }

    pub fn get_nodes(&self) -> &BTreeMap<[u8; HASH_LEN], Node<N, HW, HASH_LEN>> {
        &self.nodes
    }

    // TODO(jwall): How to specify a partial ordering for nodes in a graph?
}

impl<N, HW, const HASH_LEN: usize> Default for DAG<N, HW, HASH_LEN>
where
    N: ByteEncoder,
    HW: HashWriter<HASH_LEN>,
{
    fn default() -> Self {
        Self {
            roots: BTreeSet::new(),
            nodes: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod test;

#[cfg(all(test, feature = "proptest"))]
mod proptest;

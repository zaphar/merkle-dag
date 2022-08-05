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

/// Node comparison values. In a given Merkle DAG a Node can come `After`, `Before`, be `Equivalent`, or `Uncomparable`.
/// If the two nodes have the same id they are eqivalent. If two nodes are not part of the same sub graph within the DAG
/// then they are Uncomparable. If one node is an ancestor of another DAG then that node comes before the other. If the
/// reverse is true then that node comes after the other.
#[derive(PartialEq, Debug)]
pub enum NodeCompare {
    After,
    Before,
    Equivalent,
    Uncomparable,
}

#[derive(Debug)]
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
#[derive(Clone, Debug)]
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

    /// Get the set of root node ids.
    pub fn get_roots(&self) -> &BTreeSet<[u8; HASH_LEN]> {
        &self.roots
    }

    /// Get the map of all nodes in the DAG.
    pub fn get_nodes(&self) -> &BTreeMap<[u8; HASH_LEN], Node<N, HW, HASH_LEN>> {
        &self.nodes
    }

    /// Compare two nodes by id in the graph. If the left id is an ancestor of the right node
    /// then `returns `NodeCompare::Before`. If the right id is an ancestor of the left node
    /// then returns `NodeCompare::After`. If both id's are equal then the returns
    /// `NodeCompare::Equivalent`. If neither id are parts of the same subgraph then returns
    /// `NodeCompare::Uncomparable`.
    pub fn compare(&self, left: &[u8; HASH_LEN], right: &[u8; HASH_LEN]) -> NodeCompare {
        if left == right {
            NodeCompare::Equivalent
        } else {
            // Is left node an ancestor of right node?
            if self.search_graph(right, left) {
                NodeCompare::Before
                // is right node an ancestor of left node?
            } else if self.search_graph(left, right) {
                NodeCompare::After
            } else {
                NodeCompare::Uncomparable
            }
        }
    }

    fn search_graph(&self, root_id: &[u8; HASH_LEN], search_id: &[u8; HASH_LEN]) -> bool {
        if root_id == search_id {
            return true;
        }
        let root_node = match self.get_node_by_id(root_id) {
            Some(n) => n,
            None => {
                return false;
            }
        };
        let mut stack = vec![root_node];
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            let deps = node.dependency_ids();
            for dep in deps {
                if search_id == dep {
                    return true;
                }
                stack.push(match self.get_node_by_id(dep) {
                    Some(n) => n,
                    None => panic!("Invalid DAG STATE encountered"),
                })
            }
        }
        return false;
    }
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

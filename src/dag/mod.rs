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
//! Implementation of the MerkleDag based off of the merkle-crdt whitepaper.

use std::{collections::BTreeSet, marker::PhantomData};

use crate::{
    hash::HashWriter,
    node::Node,
    store::{Result, Store, StoreError},
};

mod iter;
pub use iter::*;

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
pub struct Merkle<S, HW>
where
    HW: HashWriter,
    S: Store<HW>,
{
    roots: BTreeSet<Vec<u8>>,
    nodes: S,
    _phantom_node: PhantomData<Node<HW>>,
}

impl<S, HW> Merkle<S, HW>
where
    HW: HashWriter,
    S: Store<HW>,
{
    /// Construct a new empty DAG. The empty DAG is also the default for a DAG.
    pub fn new(s: S) -> Self {
        Self {
            nodes: s,
            roots: Default::default(),
            _phantom_node: PhantomData,
        }
    }

    /// Add a new payload with a required set of dependency_ids. This method will construct a new node
    /// and add it to the DAG with the given payload item and dependency id set. It is idempotent for any
    /// given set of inputs.
    ///
    /// One result of not constructing and then adding nodes in this way is that we ensure that we always
    /// satisfy the implementation rule in the merkel-crdt's whitepaper.
    pub fn add_node<'a, N: Into<Vec<u8>>>(
        &'a mut self,
        item: N,
        dependency_ids: BTreeSet<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        let node = Node::<HW>::new(item.into(), dependency_ids.clone());
        let id = node.id().to_vec();
        if self.nodes.contains(id.as_slice())? {
            // We've already added this node so there is nothing left to do.
            return Ok(self
                .nodes
                .get(id.as_slice())
                .unwrap()
                .unwrap()
                .id()
                .to_vec());
        }
        let mut root_removals = Vec::new();
        for dep_id in dependency_ids.iter() {
            if !self.nodes.contains(dep_id)? {
                return Err(StoreError::NoSuchDependents);
            }
            // If any of our dependencies is in the roots pointer list then
            // we need to remove it below.
            if self.roots.contains(dep_id) {
                root_removals.push(dep_id);
            }
        }
        self.nodes.store(node)?;
        for removal in root_removals {
            self.roots.remove(removal);
        }
        self.roots.insert(id.to_vec());
        Ok(id.to_vec())
    }

    /// Check if we already have a copy of a node.
    pub fn check_for_node(&self, id: &[u8]) -> Result<bool> {
        return self.nodes.contains(id);
    }

    /// Get a node from the DAG by it's hash identifier if it exists.
    pub fn get_node_by_id(&self, id: &[u8]) -> Result<Option<Node<HW>>> {
        self.nodes.get(id)
    }

    /// Get the set of root node ids.
    pub fn get_roots(&self) -> &BTreeSet<Vec<u8>> {
        &self.roots
    }

    /// Get the map of all nodes in the DAG.
    pub fn get_nodes(&self) -> &S {
        &self.nodes
    }

    /// Compare two nodes by id in the graph. If the left id is an ancestor of the right node
    /// then `returns `NodeCompare::Before`. If the right id is an ancestor of the left node
    /// then returns `NodeCompare::After`. If both id's are equal then the returns
    /// `NodeCompare::Equivalent`. If neither id are parts of the same subgraph then returns
    /// `NodeCompare::Uncomparable`.
    pub fn compare(&self, left: &[u8], right: &[u8]) -> Result<NodeCompare> {
        Ok(if left == right {
            NodeCompare::Equivalent
        } else {
            // Is left node an ancestor of right node?
            if self.search_graph(right, left)? {
                NodeCompare::Before
                // is right node an ancestor of left node?
            } else if self.search_graph(left, right)? {
                NodeCompare::After
            } else {
                NodeCompare::Uncomparable
            }
        })
    }

    pub fn gap_fill_iter<'dag, 'iter>(
        &'dag self,
        search_nodes: BTreeSet<Vec<u8>>,
    ) -> Gap<'iter, S, HW>
    where
        'dag: 'iter,
    {
        Gap::new(self, search_nodes)
    }

    /// Find the immediate next non descendant nodes in this graph for the given `search_nodes`.
    pub fn find_next_non_descendant_nodes(
        &self,
        search_nodes: &BTreeSet<Vec<u8>>,
    ) -> Result<Vec<Node<HW>>> {
        let mut stack: Vec<Vec<u8>> = self.roots.iter().cloned().collect();
        let mut ids = BTreeSet::new();
        while !stack.is_empty() {
            let node_id = stack.pop().unwrap();
            let node = self.get_node_by_id(node_id.as_slice())?.unwrap();
            let deps = node.dependency_ids();
            if deps.len() == 0 {
                // This is a leaf node which means it's the beginning of a sub graph
                // the search_nodes_are not part of.
                ids.insert(node.id().to_owned());
            }
            for dep in deps {
                // We found one of the search roots.
                if search_nodes.contains(dep.as_slice()) {
                    // This means that the previous node is a parent of the search_roots.
                    ids.insert(node.id().to_owned());
                    continue;
                }
                stack.push(dep.to_owned())
            }
        }
        let mut result = Vec::new();
        for id in ids {
            result.push(self.get_node_by_id(id.as_slice())?.unwrap());
        }
        Ok(result)
    }

    fn search_graph(&self, root_id: &[u8], search_id: &[u8]) -> Result<bool> {
        if root_id == search_id {
            return Ok(true);
        }
        let root_node = match self.get_node_by_id(root_id)? {
            Some(n) => n,
            None => {
                return Ok(false);
            }
        };
        let mut stack = vec![root_node];
        while !stack.is_empty() {
            let node = stack.pop().unwrap();
            let deps = node.dependency_ids();
            for dep in deps {
                if search_id == dep {
                    return Ok(true);
                }
                stack.push(match self.get_node_by_id(dep)? {
                    Some(n) => n,
                    None => panic!("Invalid DAG STATE encountered"),
                })
            }
        }
        return Ok(false);
    }
}

impl<S, HW> Default for Merkle<S, HW>
where
    HW: HashWriter,
    S: Store<HW> + Default,
{
    fn default() -> Self {
        Self {
            roots: BTreeSet::new(),
            nodes: S::default(),
            _phantom_node: Default::default(),
        }
    }
}

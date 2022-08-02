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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, item: N, dependency_ids: Vec<Vec<u8>>) -> Result<(), EdgeError> {
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

    pub fn get_node(&self, id: &Vec<u8>) -> Option<&Node<N, HW>> {
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

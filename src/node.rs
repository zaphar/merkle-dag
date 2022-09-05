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
//! `Node` types for satisfying the properties necessary for a MerkleDag.

use std::{collections::BTreeSet, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::hash::HashWriter;

// NOTE(jwall): Since we enforce certain properties by construction in our DAG
// It's important that serialization isn't able to bypass that. This struct
// allows us to only serialize and deserialize the non-computable fields of a
// node.
#[derive(Serialize, Deserialize)]
struct NodeSerde {
    item: Vec<u8>,
    dependency_ids: BTreeSet<Vec<u8>>,
}

impl<HW> From<NodeSerde> for Node<HW>
where
    HW: HashWriter,
{
    fn from(ns: NodeSerde) -> Self {
        Self::new(ns.item, ns.dependency_ids)
    }
}

/// A node in a merkle DAG. Nodes are composed of a payload item and a set of dependency_ids.
/// They provide a unique identifier that is formed from the bytes of the payload as well
/// as the bytes of the dependency_ids. This is guaranteed to be the id for the same payload
/// and dependency ids every time making Nodes content-addressable.
///
/// Nodes also expose the unique content address of the item payload alone as a convenience.
///
/// Nodes are tied to a specific implementation of the HashWriter trait which is itself tied
/// to the DAG they are stored in guaranteeing that the same Hashing implementation is used
/// for each node in the DAG.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "NodeSerde")]
pub struct Node<HW>
where
    HW: HashWriter,
{
    id: Vec<u8>,
    item: Vec<u8>,
    item_id: Vec<u8>,
    dependency_ids: BTreeSet<Vec<u8>>,
    _phantom: PhantomData<HW>,
}

impl<HW> Clone for Node<HW>
where
    HW: HashWriter,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            item: self.item.clone(),
            item_id: self.item_id.clone(),
            dependency_ids: self.dependency_ids.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<HW> Node<HW>
where
    HW: HashWriter,
{
    /// Construct a new node with a payload and a set of dependency_ids.
    pub fn new<P: Into<Vec<u8>>>(item: P, dependency_ids: BTreeSet<Vec<u8>>) -> Self {
        let mut hw = HW::default();
        let item = item.into();
        // NOTE(jwall): The order here is important. Our reliable id creation must be stable
        // for multiple calls to this constructor. This means that we must *always*
        // 1. Record the `item_id` hash first.
        hw.record(item.iter().cloned());
        let item_id = hw.hash();
        // 2. Sort the dependency ids before recording them into our node id hash.
        let mut dependency_list = dependency_ids.iter().cloned().collect::<Vec<Vec<u8>>>();
        dependency_list.sort();
        // 3. record the dependency ids into our node id hash in the sorted order.
        for d in dependency_list.iter() {
            hw.record(d.iter().cloned());
        }
        Self {
            id: hw.hash(),
            item,
            item_id,
            dependency_ids,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> &[u8] {
        &self.id
    }

    pub fn item(&self) -> &[u8] {
        &self.item
    }

    pub fn item_id(&self) -> &[u8] {
        &self.item_id
    }

    pub fn dependency_ids(&self) -> &BTreeSet<Vec<u8>> {
        &self.dependency_ids
    }

    pub fn out_degree(&self) -> usize {
        self.dependency_ids.len()
    }
}

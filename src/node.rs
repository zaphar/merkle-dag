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
use std::{collections::BTreeSet, marker::PhantomData};

use crate::hash::{ByteEncoder, HashWriter};

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
pub struct Node<N, HW, const HASH_LEN: usize>
where
    N: ByteEncoder,
    HW: HashWriter<HASH_LEN>,
{
    id: [u8; HASH_LEN],
    item: N,
    item_id: [u8; HASH_LEN],
    dependency_ids: BTreeSet<[u8; HASH_LEN]>,
    _phantom: PhantomData<HW>,
}

impl<N, HW, const HASH_LEN: usize> Node<N, HW, HASH_LEN>
where
    N: ByteEncoder,
    HW: HashWriter<HASH_LEN>,
{
    /// Construct a new node with a payload and a set of dependency_ids.
    pub fn new(item: N, dependency_ids: BTreeSet<[u8; HASH_LEN]>) -> Self {
        let mut hw = HW::default();

        // NOTE(jwall): The order here is important. Our reliable id creation must be stable
        // for multiple calls to this constructor. This means that we must *always*
        // 1. Record the `item_id` hash first.
        hw.record(item.bytes().into_iter());
        let item_id = hw.hash();
        // 2. Sort the dependency ids before recording them into our node id hash.
        let mut dependency_list = dependency_ids
            .iter()
            .cloned()
            .collect::<Vec<[u8; HASH_LEN]>>();
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

    pub fn item(&self) -> &N {
        &self.item
    }

    pub fn item_id(&self) -> &[u8; HASH_LEN] {
        &self.item_id
    }

    pub fn dependency_ids(&self) -> &BTreeSet<[u8; HASH_LEN]> {
        &self.dependency_ids
    }
}

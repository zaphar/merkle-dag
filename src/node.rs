use std::marker::PhantomData;

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
use crate::hash::{ByteEncoder, HashWriter};

pub struct Node<N, HW> {
    id: Vec<u8>,
    item: N,
    dependency_ids: Vec<Vec<u8>>,
    _phantom: PhantomData<HW>,
}

impl<N, HW> Node<N, HW>
where
    N: ByteEncoder,
    HW: HashWriter,
{
    pub fn new(item: N, mut dependency_ids: Vec<Vec<u8>>) -> Self {
        let mut hw = HW::default();
        dependency_ids.sort();
        hw.record(item.bytes().into_iter());
        for d in dependency_ids.iter() {
            hw.record(d.iter().cloned());
        }
        Self {
            id: hw.hash(),
            item,
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

    pub fn dependency_ids(&self) -> &Vec<Vec<u8>> {
        &self.dependency_ids
    }
}

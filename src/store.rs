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
use std::collections::BTreeMap;

use crate::{hash::HashWriter, node::Node};

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone)]
pub enum StoreError {
    StoreFailure(String),
    NoSuchDependents,
}

pub trait Store<HW, const HASH_LEN: usize>: Default
where
    HW: HashWriter<HASH_LEN>,
{
    fn contains(&self, id: &[u8; HASH_LEN]) -> Result<bool>;
    fn get(&self, id: &[u8; HASH_LEN]) -> Result<Option<Node<HW, HASH_LEN>>>;
    fn store(&mut self, node: Node<HW, HASH_LEN>) -> Result<()>;
}

impl<HW, const HASH_LEN: usize> Store<HW, HASH_LEN> for BTreeMap<[u8; HASH_LEN], Node<HW, HASH_LEN>>
where
    HW: HashWriter<HASH_LEN>,
{
    fn contains(&self, id: &[u8; HASH_LEN]) -> Result<bool> {
        Ok(self.contains_key(id))
    }

    fn get(&self, id: &[u8; HASH_LEN]) -> Result<Option<Node<HW, HASH_LEN>>> {
        Ok(self.get(id).cloned())
    }

    fn store(&mut self, node: Node<HW, HASH_LEN>) -> Result<()> {
        self.insert(node.id().clone(), node);
        Ok(())
    }
}

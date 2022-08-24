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
//! Module implementing a store interface using LevelDB for a MerkleDag.
//! Requires the `rusty-leveldb` interface.

use std::cell::RefCell;
use std::path::Path;

use crate::{
    hash::HashWriter,
    node::Node,
    store::{Result, Store, StoreError},
};

use ciborium;
use rusty_leveldb;

/// A `Store` implementation using the rusty-leveldb port of leveldb.
/// The Default implementation of this `Default::default()` is an in-memory
/// implementation of the store.
pub struct LevelStore {
    store: RefCell<rusty_leveldb::DB>,
}

impl LevelStore {
    pub fn open<P: AsRef<Path>>(path: P) -> std::result::Result<Self, rusty_leveldb::Status> {
        let opts = Default::default();
        Ok(Self {
            store: RefCell::new(rusty_leveldb::DB::open(path, opts)?),
        })
    }
}

impl<HW> Store<HW> for LevelStore
where
    HW: HashWriter,
{
    fn contains(&self, id: &[u8]) -> Result<bool> {
        Ok(self.store.borrow_mut().get(id).is_some())
    }

    fn get(&self, id: &[u8]) -> Result<Option<Node<HW>>> {
        Ok(match self.store.borrow_mut().get(id) {
            Some(bs) => ciborium::de::from_reader(bs.as_slice())
                .map_err(|e| StoreError::StoreFailure(format!("Invalid serialization {:?}", e)))?,
            None => None,
        })
    }

    fn store(&mut self, node: Node<HW>) -> Result<()> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&node, &mut buf).unwrap();
        self.store.borrow_mut().put(node.id(), &buf)?;
        Ok(())
    }
}

impl From<rusty_leveldb::Status> for StoreError {
    fn from(status: rusty_leveldb::Status) -> Self {
        StoreError::StoreFailure(format!("{}", status))
    }
}

impl Default for LevelStore {
    fn default() -> Self {
        Self {
            store: RefCell::new(
                rusty_leveldb::DB::open("memory", rusty_leveldb::in_memory()).unwrap(),
            ),
        }
    }
}

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
//! Module implementing a [Store] interface using rocksdb for a [Merkle Dag](crate::dag::Merkle).
//! Requires the `rocksdb` feature to be enabled.

use std::path::Path;

use crate::{
    hash::HashWriter,
    node::Node,
    store::{Result as StoreResult, Store, StoreError},
};

use ciborium;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options, SingleThreaded, ThreadMode};

pub type Result<T> = std::result::Result<T, rocksdb::Error>;

/// A Rocksdb `Store` implementation generic over the single and multithreaded
/// versions.
pub struct RocksStore<TM>
where
    TM: ThreadMode,
{
    store: DBWithThreadMode<TM>,
}

/// Type alias for a [RocksStore<SingleThreaded>].
pub type SingleThreadedRocksStore = RocksStore<SingleThreaded>;
/// Type alias for a [RocksStore<Multithreaded>].
pub type MultiThreadedRocksStore = RocksStore<MultiThreaded>;

impl<TM> RocksStore<TM>
where
    TM: ThreadMode,
{
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let opts = Options::default();
        Self::open_with_opts(path, &opts)
    }

    pub fn open_with_opts<P: AsRef<Path>>(path: P, opts: &Options) -> Result<Self> {
        Ok(Self {
            store: DBWithThreadMode::<TM>::open(&opts, path)?,
        })
    }
}

impl<TM, HW> Store<HW> for RocksStore<TM>
where
    TM: ThreadMode,
    HW: HashWriter,
{
    fn contains(&self, id: &[u8]) -> StoreResult<bool> {
        Ok(self
            .store
            .get(id)
            .map_err(|e| StoreError::StoreFailure(format!("{:?}", e)))?
            .is_some())
    }

    fn get(&self, id: &[u8]) -> StoreResult<Option<Node<HW>>> {
        Ok(
            match self
                .store
                .get(id)
                .map_err(|e| StoreError::StoreFailure(format!("{:?}", e)))?
            {
                Some(bs) => ciborium::de::from_reader(bs.as_slice()).map_err(|e| {
                    StoreError::StoreFailure(format!("Invalid serialization {:?}", e))
                })?,
                None => None,
            },
        )
    }

    fn store(&mut self, node: Node<HW>) -> StoreResult<()> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&node, &mut buf).unwrap();
        self.store.put(node.id(), &buf)?;
        Ok(())
    }
}

impl From<rocksdb::Error> for StoreError {
    fn from(err: rocksdb::Error) -> Self {
        StoreError::StoreFailure(format!("{}", err))
    }
}

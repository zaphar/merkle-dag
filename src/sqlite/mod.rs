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
use std::path::Path;

use crate::{
    hash::HashWriter,
    node::Node,
    store::{Result as StoreResult, Store, StoreError},
};

use ciborium;
use rusqlite::{self, OptionalExtension};

pub struct SqliteStore {
    conn: rusqlite::Connection,
}

impl SqliteStore {
    pub fn connect<P: AsRef<Path>>(path: P) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            conn: rusqlite::Connection::open(path)?,
        })
    }

    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let me = Self {
            conn: rusqlite::Connection::open_in_memory()?,
        };
        me.init_db()?;
        Ok(me)
    }

    pub fn init_db(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "BEGIN;
        CREATE TABLE content_store(content_id BLOB PRIMARY KEY, node BLOB NOT NULL);
        COMMIT;",
        )?;
        Ok(())
    }
}

impl<HW> Store<HW> for SqliteStore
where
    HW: HashWriter,
{
    fn contains(&self, id: &[u8]) -> StoreResult<bool> {
        let val: Option<Vec<u8>> = self
            .conn
            .query_row(
                "select node from content_store where content_id = ?",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(val.is_some())
    }

    fn get(&self, id: &[u8]) -> StoreResult<Option<Node<HW>>> {
        let result: Option<Vec<u8>> = self
            .conn
            .query_row(
                "select node from content_store where content_id = ?",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match result {
            Some(bs) => ciborium::de::from_reader(bs.as_slice())
                .map_err(|e| StoreError::StoreFailure(format!("Invalid serialization {:?}", e)))?,
            None => None,
        })
    }

    fn store(&mut self, node: Node<HW>) -> StoreResult<()> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&node, &mut buf).unwrap();
        self.conn.execute(
            "insert into content_store (content_id, node) values (?, ?)",
            [node.id(), buf.as_slice()],
        )?;
        Ok(())
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::StoreFailure(format!("{:?}", e))
    }
}

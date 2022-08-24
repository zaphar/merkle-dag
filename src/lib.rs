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
//! A merkle dag along the lines of the merkle-crdt whitepaper.

#[cfg(feature = "blake2")]
pub mod blake2;
pub mod dag;
pub mod hash;
#[cfg(feature = "rusty-leveldb")]
pub mod leveldb;
pub mod node;
pub mod prelude;
#[cfg(feature = "rocksdb")]
pub mod rocksdb;
pub mod store;

#[cfg(test)]
mod test;

#[cfg(all(test, feature = "proptest"))]
mod proptest;

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

use std::hash::Hasher;

/// Utility Trait to specify that payloads must be serializable into bytes.
pub trait ByteEncoder {
    fn bytes(&self) -> Vec<u8>;
}

/// Utility Trait to specify the hashing algorithm and provide a common
/// interface for that algorithm to provide. This interface is expected to
/// be stateful.
pub trait HashWriter<const LEN: usize>: Default {
    /// Record bytes from an iterator into our hash algorithm.
    fn record<I: Iterator<Item = u8>>(&mut self, bs: I);

    /// Provide the current hash value based on the bytes that have so far been recorded.
    /// It is expected that you can call this method multiple times while recording the
    /// the bytes for input into the hash.
    fn hash(&self) -> [u8; LEN];
}

impl<H> HashWriter<8> for H
where
    H: Hasher + Default,
{
    fn record<I: Iterator<Item = u8>>(&mut self, iter: I) {
        let bytes = iter.collect::<Vec<u8>>();
        self.write(bytes.as_slice());
    }

    fn hash(&self) -> [u8; 8] {
        self.finish().to_le_bytes()
    }
}

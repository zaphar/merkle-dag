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
use crate::hash::*;
use blake2::digest::Digest;
pub use blake2::{Blake2b512, Blake2s256};

macro_rules! hash_writer_impl {
    ($tname:ident, $size:expr) => {
        impl HashWriter<$size> for $tname {
            fn record<I: Iterator<Item = u8>>(&mut self, bs: I) {
                let vec: Vec<u8> = bs.collect();
                self.update(&vec);
            }

            fn hash(&self) -> [u8; $size] {
                let mut out: [u8; $size] = Default::default();
                // This is gross but Blake2 doesn't support the
                // non consuming version of this.
                let mut arr = self.clone().finalize();
                arr.swap_with_slice(&mut out);
                out
            }
        }
    };
}

hash_writer_impl!(Blake2b512, 8);
hash_writer_impl!(Blake2s256, 4);

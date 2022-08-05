<!--
 Copyright 2022 Jeremy Wall (Jeremy@marzhilsltudios.com)
 
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 
     http://www.apache.org/licenses/LICENSE-2.0
 
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->

# A generic implementation of a Merkle DAG

This is an exploration of the paper [merkle-crdts](https://arxiv.org/pdf/2004.00107.pdf) as well,
if I'm honest, as an excuse to use [proptests](https://crates.io/crate/proptest).

The proptest assertions are hidden behind the `proptest` feature since they can take longer to run than
the standard tests. To run them: `cargo test --features proptest`
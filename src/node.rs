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

use serde::{de::Visitor, ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::HashWriter;

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
#[derive(Debug, PartialEq, Eq)]
pub struct Node<HW, const HASH_LEN: usize>
where
    HW: HashWriter<HASH_LEN>,
{
    id: [u8; HASH_LEN],
    item: Vec<u8>,
    item_id: [u8; HASH_LEN],
    dependency_ids: BTreeSet<[u8; HASH_LEN]>,
    _phantom: PhantomData<HW>,
}

impl<HW, const HASH_LEN: usize> Clone for Node<HW, HASH_LEN>
where
    HW: HashWriter<HASH_LEN>,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            item: self.item.clone(),
            item_id: self.item_id.clone(),
            dependency_ids: self.dependency_ids.clone(),
            _phantom: PhantomData,
        }
    }
}

fn coerce_non_const_generic_set<const HASH_LEN: usize>(
    set: &BTreeSet<[u8; HASH_LEN]>,
) -> BTreeSet<&[u8]> {
    let mut coerced_item = BTreeSet::new();
    for arr in set {
        coerced_item.insert(arr.as_slice());
    }
    coerced_item
}

impl<HW, const HASH_LEN: usize> Serialize for Node<HW, HASH_LEN>
where
    HW: HashWriter<HASH_LEN>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut structor = serializer.serialize_struct("Node", 4)?;
        structor.serialize_field("item", &self.item)?;
        structor.serialize_field(
            "dependency_ids",
            &coerce_non_const_generic_set(&self.dependency_ids),
        )?;
        structor.end()
    }
}

fn coerce_const_generic_array<const HASH_LEN: usize>(
    slice: &[u8],
) -> Result<[u8; HASH_LEN], String> {
    let mut coerced_item: [u8; HASH_LEN] = [0; HASH_LEN];
    if slice.len() > coerced_item.len() {
        return Err(format!(
            "Expected slice of length: {} but got slice of length: {}",
            coerced_item.len(),
            slice.len()
        ));
    } else {
        coerced_item.copy_from_slice(slice);
    }
    Ok(coerced_item)
}

fn coerce_const_generic_set<const HASH_LEN: usize>(
    set: BTreeSet<&[u8]>,
) -> Result<BTreeSet<[u8; HASH_LEN]>, String> {
    let mut coerced_item = BTreeSet::new();
    for slice in set {
        coerced_item.insert(coerce_const_generic_array(slice)?);
    }
    Ok(coerced_item)
}

impl<'de, HW, const HASH_LEN: usize> Deserialize<'de> for Node<HW, HASH_LEN>
where
    HW: HashWriter<HASH_LEN>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            Item,
            Dependency_Ids,
        }

        struct NodeVisitor<HW, const HASH_LEN: usize>(PhantomData<HW>);

        impl<'de, HW, const HASH_LEN: usize> Visitor<'de> for NodeVisitor<HW, HASH_LEN>
        where
            HW: HashWriter<HASH_LEN>,
        {
            type Value = Node<HW, HASH_LEN>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Node")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let item = seq
                    .next_element::<Vec<u8>>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let dependency_ids: BTreeSet<[u8; HASH_LEN]> = coerce_const_generic_set(
                    seq.next_element::<BTreeSet<&[u8]>>()?
                        .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?,
                )
                .map_err(serde::de::Error::custom)?;
                Ok(Self::Value::new(item, dependency_ids))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut item: Option<Vec<u8>> = None;
                let mut dependency_ids: Option<BTreeSet<[u8; HASH_LEN]>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Item => {
                            if item.is_some() {
                                return Err(serde::de::Error::duplicate_field("item"));
                            } else {
                                item = Some(map.next_value()?);
                            }
                        }
                        Field::Dependency_Ids => {
                            if dependency_ids.is_some() {
                                return Err(serde::de::Error::duplicate_field("dependency_ids"));
                            } else {
                                dependency_ids = Some(
                                    coerce_const_generic_set(map.next_value()?)
                                        .map_err(serde::de::Error::custom)?,
                                );
                            }
                        }
                    }
                }
                let item = item.ok_or_else(|| serde::de::Error::missing_field("item"))?;
                let dependency_ids = dependency_ids
                    .ok_or_else(|| serde::de::Error::missing_field("dependency_ids"))?;

                Ok(Self::Value::new(item, dependency_ids))
            }
        }

        const FIELDS: &'static [&'static str] = &["item", "dependency_ids"];
        deserializer.deserialize_struct("Node", FIELDS, NodeVisitor::<HW, HASH_LEN>(PhantomData))
    }
}

impl<HW, const HASH_LEN: usize> Node<HW, HASH_LEN>
where
    HW: HashWriter<HASH_LEN>,
{
    /// Construct a new node with a payload and a set of dependency_ids.
    pub fn new<P: Into<Vec<u8>>>(item: P, dependency_ids: BTreeSet<[u8; HASH_LEN]>) -> Self {
        let mut hw = HW::default();
        let item = item.into();
        // NOTE(jwall): The order here is important. Our reliable id creation must be stable
        // for multiple calls to this constructor. This means that we must *always*
        // 1. Record the `item_id` hash first.
        hw.record(item.iter().cloned());
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

    pub fn id(&self) -> &[u8; HASH_LEN] {
        &self.id
    }

    pub fn item(&self) -> &[u8] {
        &self.item
    }

    pub fn item_id(&self) -> &[u8; HASH_LEN] {
        &self.item_id
    }

    pub fn dependency_ids(&self) -> &BTreeSet<[u8; HASH_LEN]> {
        &self.dependency_ids
    }
}

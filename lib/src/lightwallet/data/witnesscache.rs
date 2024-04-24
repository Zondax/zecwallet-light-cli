use std::usize;

use zcash_primitives::merkle_tree::IncrementalWitness;
use zcash_primitives::sapling::Node;

use crate::lightclient::blaze::fixed_size_buffer::FixedSizeBuffer;

#[derive(Clone)]
// todo: this should be part of the light client
pub(crate) struct WitnessCache {
    pub(crate) witnesses: Vec<IncrementalWitness<Node>>,
    pub(crate) top_height: u64,
}

impl WitnessCache {
    pub fn new(
        witnesses: Vec<IncrementalWitness<Node>>,
        top_height: u64,
    ) -> Self {
        Self { witnesses, top_height }
    }

    pub fn empty() -> Self {
        Self { witnesses: vec![], top_height: 0 }
    }

    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }

    pub fn clear(&mut self) {
        self.witnesses.clear();
    }

    pub fn get(
        &self,
        i: usize,
    ) -> Option<&IncrementalWitness<Node>> {
        self.witnesses.get(i)
    }

    #[cfg(test)]
    pub fn get_from_last(
        &self,
        i: usize,
    ) -> Option<&IncrementalWitness<Node>> {
        self.witnesses.get(self.len() - i - 1)
    }

    pub fn last(&self) -> Option<&IncrementalWitness<Node>> {
        self.witnesses.last()
    }

    pub fn into_fsb(
        self,
        fsb: &mut FixedSizeBuffer<IncrementalWitness<Node>>,
    ) {
        self.witnesses
            .into_iter()
            .for_each(|w| fsb.push(w));
    }

    pub fn pop(
        &mut self,
        at_height: u64,
    ) {
        while !self.witnesses.is_empty() && self.top_height >= at_height {
            self.witnesses.pop();
            self.top_height -= 1;
        }
    }

    // pub fn get_as_string(&self, i: usize) -> String {
    //     if i >= self.witnesses.len() {
    //         return "".to_string();
    //     }

    //     let mut buf = vec![];
    //     self.get(i).unwrap().write(&mut buf).unwrap();
    //     return hex::encode(buf);
    // }
}

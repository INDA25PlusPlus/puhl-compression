use core::panic;
use bitvec::prelude::*;
use std::{cmp::Ordering, collections::BinaryHeap, io::{self, Read}};

#[derive(Eq)]
struct HuffmanNode {
    frequency: usize,
    byte: Option<u8>,
    l_child: Option<Box<HuffmanNode>>,
    r_child: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    // TODO: Maybe take a box directly to avoid overhead of moving the node here
    fn new(frequency: usize, byte: Option<u8>, l_child: Option<Box<HuffmanNode>>, r_child: Option<Box<HuffmanNode>>) -> Self {
        HuffmanNode { l_child, r_child, frequency, byte }
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency && self.byte == other.byte
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.frequency.cmp(&self.frequency) {
            Ordering::Equal => self.byte.cmp(&other.byte), // Tiebreak by byte value
            ord => ord,
        }
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct HuffmanEncoder;

impl HuffmanEncoder {
    pub fn build(data: &[u8]) -> Option<Box<HuffmanNode>> {
        let frequency = Self::count_frequency(data);
        
        let mut heap: BinaryHeap<_> = frequency.iter()
            .enumerate()
            .filter(|&(_, &freq)|
                freq != 0
            )
            .map(|(byte, &freq)| 
                Box::new(
                    HuffmanNode::new(freq, Some(byte as u8), None, None)
                )
            )
            .collect();

        if heap.len() == 0 {
            return None;
        }
        if heap.len() == 1 {
            panic!("Data with single byte not supported yet")
        }

        while heap.len() > 1 {
            let node1 = heap.pop().unwrap();
            let node2 = heap.pop().unwrap();

            let parent = Box::new(HuffmanNode::new(
                node1.frequency + node2.frequency,
                None,
                Some(node1),
                Some(node2),
            ));

            heap.push(parent);
        }

        let root = heap.pop().unwrap();
        // Fix incase only one node in heap
        // let root = match root.byte {
        //     Some(byte) => root,
        //     None => Box::new(HuffmanNode::new(
        //         root.frequency,
        //         None,
        //         Some(root),
        //         None
        //     )),
        // };

        Some(root)
    }

    fn count_frequency(data: &[u8]) -> [usize; 256] {
        let mut frequency = [0 as usize; 256];
        data.iter().for_each(|&byte| {
            frequency[byte as usize] += 1;
        });
        frequency
    }
}

struct HuffmanSeralizer {}

impl HuffmanSeralizer {
    pub fn serialize(root: &HuffmanNode, data: &[u8]) -> BitVec<u8, Msb0> {
        let mut content = BitVec::<u8, Msb0>::new();
        Self::serialize_tree(root, &mut content);
        let content_len: u64 = content.len() as u64;
        let data_len: u64 = data.len() as u64;

        let mut header = BitVec::<u8, Msb0>::new();
        header.extend_from_bitslice(b"JP".view_bits::<Msb0>());
        header.extend_from_bitslice(content_len.to_be_bytes().view_bits::<Msb0>());
        header.extend_from_bitslice(data_len.to_be_bytes().view_bits::<Msb0>());
        header.extend_from_bitslice(&content);

        let mut table: [BitVec<u8, Msb0>; 256] = std::array::from_fn(|_| BitVec::new());
        Self::build_encoding_table(root, &mut table, &mut BitVec::<u8, Msb0>::new());

        Self::serialize_encoded_data(&table, data, &mut header);

        header
    }

    fn serialize_tree(root: &HuffmanNode, bitstream: &mut BitVec<u8, Msb0>) {
        match root.byte {
            Some(byte) => {
                bitstream.push(true);
                bitstream.extend_from_bitslice(byte.view_bits::<Msb0>());
            },
            None => { 
                bitstream.push(false);
                Self::serialize_tree(root.l_child.as_ref().unwrap(), bitstream);
                Self::serialize_tree(root.r_child.as_ref().unwrap(), bitstream);
            }
        };
    }

    fn build_encoding_table(root: &HuffmanNode, table: &mut [BitVec<u8, Msb0>; 256], current_encoding: &mut BitVec<u8, Msb0>) {
        match root.byte {
            Some(byte) => {
                table[byte as usize] = current_encoding.clone();
            },
            None => {
                current_encoding.push(false);
                Self::build_encoding_table(root.l_child.as_ref().unwrap(), table, current_encoding);
                current_encoding.pop().unwrap();

                current_encoding.push(true);
                Self::build_encoding_table(root.r_child.as_ref().unwrap(), table, current_encoding);
                current_encoding.pop().unwrap();
            }
        }    
    }

    fn serialize_encoded_data(table: &[BitVec<u8, Msb0>; 256], data: &[u8], bitstream: &mut BitVec<u8, Msb0>) {
        for &byte in data {
            bitstream.extend_from_bitslice(&table[byte as usize]);
        }
    }
}

fn main() {
    let mut input = vec![];
    let error = io::stdin().read_to_end(&mut input);
    match error {
        Err(e) => panic!("{}", e),
        Ok(_) => (),
    }

    
}

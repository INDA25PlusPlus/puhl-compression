use bitvec::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::{self, Read};

#[derive(Eq)]
pub struct HuffmanNode {
    frequency: usize,
    byte: Option<u8>,
    l_child: Option<Box<HuffmanNode>>,
    r_child: Option<Box<HuffmanNode>>,
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency && self.byte == other.byte
    }
}

// Reverse the order so the the huffman heap becomes a min heap
impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.frequency.cmp(&self.frequency) {
            Ordering::Equal => self.byte.cmp(&other.byte),
            ord => ord,
        }
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HuffmanNode {
    fn new_leaf(frequency: usize, byte: u8) -> Self {
        Self {
            frequency,
            byte: Some(byte),
            l_child: None,
            r_child: None,
        }
    }

    fn new_internal(left: HuffmanNode, right: HuffmanNode) -> Self {
        Self {
            frequency: left.frequency + right.frequency,
            byte: None,
            l_child: Some(Box::new(left)),
            r_child: Some(Box::new(right)),
        }
    }
}

pub struct HuffmanTree {
    root: Box<HuffmanNode>,
}

impl HuffmanTree {
    pub fn build(data: &[u8]) -> Self {
        let mut frequency = [0usize; 256];
        for &byte in data {
            frequency[byte as usize] += 1;
        }

        let mut heap: BinaryHeap<HuffmanNode> = frequency
            .iter()
            .enumerate()
            .filter(|&(_, &freq)| freq != 0)
            .map(|(byte, &freq)| HuffmanNode::new_leaf(freq, byte as u8))
            .collect();

        // If the file is empty, make a dummy node
        if heap.is_empty() {
            heap.push(HuffmanNode::new_leaf(0, 0)); 
        }

        // Edge case where data only has one unique character
        if heap.len() == 1 {
            let single_node = heap.pop().unwrap();
            let dummy_node = HuffmanNode::new_leaf(0, 0);
            heap.push(HuffmanNode::new_internal(single_node, dummy_node));
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            heap.push(HuffmanNode::new_internal(left, right));
        }

        Self {
            root: Box::new(heap.pop().unwrap()),
        }
    }

    pub fn serialize(&self) -> BitVec<u8, Msb0> {
        let mut bitstream = BitVec::<u8, Msb0>::new();
        Self::serialize_recursive(&self.root, &mut bitstream);
        bitstream
    }

    pub fn get_encoding_table(&self) -> Vec<BitVec<u8, Msb0>> {
        let mut table = vec![BitVec::<u8, Msb0>::new(); 256];
        Self::build_table_recursive(&self.root, &mut table, &mut BitVec::new());
        table
    }

    fn serialize_recursive(node: &HuffmanNode, bitstream: &mut BitVec<u8, Msb0>) {
        if let Some(byte) = node.byte {
            bitstream.push(true);
            bitstream.extend_from_bitslice(byte.view_bits::<Msb0>());
        } else {
            bitstream.push(false);
            if let Some(ref left) = node.l_child {
                Self::serialize_recursive(left, bitstream);
            }
            if let Some(ref right) = node.r_child {
                Self::serialize_recursive(right, bitstream);
            }
        }
    }

    fn build_table_recursive(
        node: &HuffmanNode,
        table: &mut [BitVec<u8, Msb0>],
        current_encoding: &mut BitVec<u8, Msb0>,
    ) {
        if let Some(byte) = node.byte {
            table[byte as usize] = current_encoding.clone();
        } else {
            if let Some(ref left) = node.l_child {
                current_encoding.push(false);
                Self::build_table_recursive(left, table, current_encoding);
                current_encoding.pop();
            }
            if let Some(ref right) = node.r_child {
                current_encoding.push(true);
                Self::build_table_recursive(right, table, current_encoding);
                current_encoding.pop();
            }
        }
    }
}

pub struct HuffmanArchive;

impl HuffmanArchive {
    pub fn compress(data: &[u8], tree: &HuffmanTree) -> BitVec<u8, Msb0> {
        let tree_bits = tree.serialize();
        let encoding_table = tree.get_encoding_table();

        let tree_len = tree_bits.len() as u64;
        let data_len = data.len() as u64;

        let mut archive = BitVec::<u8, Msb0>::new();
        
        // Magic Number
        archive.extend_from_bitslice(b"JP".view_bits::<Msb0>());
        
        // Metadata
        archive.extend_from_bitslice(tree_len.to_be_bytes().view_bits::<Msb0>());
        archive.extend_from_bitslice(data_len.to_be_bytes().view_bits::<Msb0>());
        
        // Tree Structure
        archive.extend_from_bitslice(&tree_bits);

        // Encoded Data
        for &byte in data {
            archive.extend_from_bitslice(&encoding_table[byte as usize]);
        }

        archive
    }
}

fn main() {
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).expect("Failed to read from stdin");

    let tree = HuffmanTree::build(&input);
    let compressed_data = HuffmanArchive::compress(&input, &tree);
}
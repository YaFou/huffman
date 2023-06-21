use std::{
    cmp::Reverse,
    collections::{BinaryHeap, LinkedList},
    fs::File,
    io::{Error, Read, Write},
};

use hashbrown::HashMap;

#[derive(PartialEq, Eq, Debug)]
enum HuffmanTree {
    Char(char),
    Node(Box<HuffmanTree>, Box<HuffmanTree>),
}

#[derive(PartialEq, Eq)]
struct HuffmanWeight {
    tree: HuffmanTree,
    weight: u32,
}

impl PartialOrd for HuffmanWeight {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight.partial_cmp(&other.weight)
    }
}

impl Ord for HuffmanWeight {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight.cmp(&other.weight)
    }
}

type BitVec = Vec<bool>;

fn nb_occ(string: String) -> HashMap<char, u32> {
    let mut map = HashMap::new();

    string.chars().into_iter().for_each(|c| {
        match map.get(&c) {
            None => map.insert(c, 1),
            Some(k) => map.insert(c, k + 1),
        };
    });

    map
}

fn make_huffman_tree(map: HashMap<char, u32>) -> HuffmanTree {
    let mut heap = BinaryHeap::new();

    map.into_iter().for_each(|(c, i)| {
        heap.push(Reverse(Box::new(HuffmanWeight {
            tree: HuffmanTree::Char(c),
            weight: i,
        })))
    });

    while heap.len() > 1 {
        let tree1 = heap.pop().unwrap().0;
        let tree2 = heap.pop().unwrap().0;
        let tree = HuffmanWeight {
            tree: HuffmanTree::Node(Box::new(tree1.tree), Box::new(tree2.tree)),
            weight: tree1.weight + tree2.weight,
        };
        heap.push(Reverse(Box::new(tree)));
    }

    heap.pop().unwrap().0.tree
}

fn make_huffman_map<'a>(tree: HuffmanTree) -> HashMap<char, Vec<bool>> {
    let mut map = HashMap::new();
    let mut stack = LinkedList::new();
    stack.push_front((tree, Vec::new()));

    while !stack.is_empty() {
        let (tree, code) = stack.pop_front().unwrap();
        match tree {
            HuffmanTree::Char(c) => {
                map.insert(c, code);
            }
            HuffmanTree::Node(tree1, tree2) => {
                let mut code1 = code.clone();
                code1.push(false);
                stack.push_front((*tree1, code1));

                let mut code2 = code.clone();
                code2.push(true);
                stack.push_front((*tree2, code2));
            }
        }
    }

    return map;
}

fn bit_vec_to_string(code: BitVec) -> String {
    code.iter()
        .map(|b| match b {
            false => '0',
            true => '1',
        })
        .collect()
}

fn string_to_bit_vec(code: String) -> BitVec {
    let mut v = Vec::new();

    code.chars().for_each(|c| match c {
        '0' => v.push(false),
        '1' => v.push(true),
        _ => {
            println!("PANIC {}", c as u8);
            panic!();
        }
    });

    v
}

fn byte_to_bit_vec(b: u8) -> BitVec {
    let mut v = Vec::new();
    let mut n = 128;
    let mut r = b;
    for _ in 0..8 {
        v.push(r / n == 1);
        r = r % n;
        n /= 2;
    }

    v
}

pub fn encode_file(in_file: String, out_file: String) -> Result<(), Error> {
    let mut file = File::open(in_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let map = make_huffman_map(make_huffman_tree(nb_occ(contents.clone())));

    let mut lines = Vec::with_capacity(map.len() + 1);
    let mut file = File::create(out_file)?;
    lines.push(format!("{} {}", contents.chars().count(), map.len()));

    for (c, v) in map.iter() {
        lines.push(format!("{} {}", c, bit_vec_to_string(v.to_vec())));
    }

    writeln!(file, "{}", lines.join("\n"))?;
    let mut bytes = Vec::new();
    contents.chars().for_each(|c| {
        bytes.append(&mut map.get(&c).unwrap().clone());
    });

    for _ in 1..(9 - bytes.len() % 8) {
        bytes.push(false);
    }

    let bytes: Vec<u8> = bytes
        .chunks(8)
        .map(|chunk| {
            let mut carry = 0;
            let mut m = 1;
            for i in (0..8).rev() {
                if chunk[i] {
                    carry += m;
                }
                if i > 0 {
                    m *= 2;
                }
            }
            carry
        })
        .collect();

    file.write(&bytes[..])?;

    Ok(())
}

pub fn decode_file(in_file: String, out_file: String) -> Result<(), Error> {
    let mut file = File::open(in_file)?;
    let mut encoded = Vec::new();
    file.read_to_end(&mut encoded)?;
    let encoded = unsafe { String::from_utf8_unchecked(encoded) };

    let lines: Vec<String> = encoded.split('\n').map(String::from).collect();

    let first_line = lines[0].split_once(' ').unwrap();
    let chars_count: u32 = first_line.0.parse().unwrap();
    let distinct_chars_count: u32 = first_line.1.parse().unwrap();

    let mut map = HashMap::with_capacity(distinct_chars_count as usize);
    let mut line_index = 1;
    let mut chars_processed = 0;
    let mut min_vec_length = usize::MAX;

    while chars_processed < distinct_chars_count {
        if lines[line_index] == "" {
            line_index += 1;
            let v = string_to_bit_vec(lines[line_index].trim().to_string());
            min_vec_length = min_vec_length.min(v.len());
            map.insert(v, '\n');
        } else {
            let c = lines[line_index].chars().nth(0).unwrap();
            let (_, code) = lines[line_index].split_at(2);
            let v = string_to_bit_vec(code.trim().to_string());
            min_vec_length = min_vec_length.min(v.len());
            map.insert(v, c);
        }
        line_index += 1;
        chars_processed += 1;
    }

    let payload_string = lines[line_index..].join("\n");
    let payload = payload_string.as_bytes();
    let mut code = Vec::new();

    payload.iter().for_each(|b| {
        code.append(&mut byte_to_bit_vec(*b));
    });

    let mut file = File::create(out_file)?;
    let mut start = 0;
    let mut length = min_vec_length;
    let mut l = 0;
    let mut out = Vec::with_capacity(chars_count as usize);

    while start < code.len() && l < chars_count {
        let chunk = &code[start..(start + length)];

        match map.get(chunk) {
            None => length += 1,
            Some(c) => {
                out.push(*c);
                start = start + length;
                length = min_vec_length;
                l += 1;
            }
        }
    }

    write!(file, "{}", out.iter().collect::<String>())?;

    Ok(())
}

use std::{cell::RefCell, collections::{HashMap, hash_map::Iter}, fmt::{Display}, ops::AddAssign, rc::Rc, vec};

type RefHuffmanTree = Rc<RefCell<HuffmanTree>>;
type Weight = u64;

/// 哈夫曼树
pub struct HuffmanTree {
    pub value: Option<char>,
    pub weight: Weight,
    pub parent: Option<RefHuffmanTree>,
    pub left: Option<RefHuffmanTree>,
    pub right: Option<RefHuffmanTree>,
}

impl HuffmanTree {
    pub fn new() -> Self {
        Self {
            value: None,
            weight: 0,
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn build(char_weight: CharWeightMap) -> RefHuffmanTree
    {
        // 原始结点数量
        let n = char_weight.len();
        // 构建完整哈夫曼树总共需要的结点数量
        let total = 2 * n - 1;
        // 初始化所有结点
        let vec = (0..total)
            .map(|_| Rc::new(RefCell::new(Self::new())))
            .collect::<Vec<Rc<RefCell<HuffmanTree>>>>();

        // 字符结点赋值
        char_weight.iter()
            .enumerate()
            .into_iter()
            .for_each(|(index, (ch, weight))| {
                // println!("{}: {} ({})", index, &weight, ch);
                vec[index].borrow_mut().value = Some(*ch);
                vec[index].borrow_mut().weight = *weight;
            });

        for index in n..total {
            // 找到 [0, index-1] 中权重最小的结点
            let m1 = Self::find_min(&vec[..index]).unwrap();
            // 标记父结点为 index 上的结点，下次就不会找到这个
            m1.borrow_mut().parent = Some(vec[index].clone());
            // 找到 [0, index-1] 中权重第二小的结点
            let m2 = Self::find_min(&vec[..index]).unwrap();
            // 标记该结点的父结点为 index 上的结点。
            m2.borrow_mut().parent = Some(vec[index].clone());

            let w1 = m1.as_ref().borrow().weight;
            let w2 = m2.as_ref().borrow().weight;
            let weight = w1 + w2;

            vec[index].borrow_mut().weight = weight;
            vec[index].borrow_mut().left = Some(m1.clone());
            vec[index].borrow_mut().right = Some(m2.clone());
        }
        // 最后一个结点即为构建好的完整哈夫曼树
        vec.last().unwrap().clone()
    }

    /// 获取最小的值
    fn find_min(tree_slice: &[Rc<RefCell<HuffmanTree>>]) -> Option<Rc<RefCell<HuffmanTree>>> {
        let mut min = Weight::MAX;
        let mut result = None;
        for tree in tree_slice {
            let tree_cell = tree.as_ref();
            if tree_cell.borrow().parent.is_none() && tree_cell.borrow().weight < min {
                min = tree_cell.borrow().weight;
                result = Some(tree.clone());
            }
        }
        result
    }
}

/// 字符权重
pub struct CharWeightMap {
    pub inner: HashMap<char, Weight>
}

impl CharWeightMap {
    pub fn build(input: &String) -> Self {
        let mut map = HashMap::new();
        for (_, c) in input.char_indices() {
            map.entry(c).or_insert(0).add_assign(1);
        }
        Self { inner: map }
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn iter(&self) -> Iter<char, Weight> {
        self.inner.iter()  
    }
}

/// 字符二进制映射，表示字符对应的二进制位，可用 bitvec 替代
pub struct HuffmanBinaryMap {
    pub inner: HashMap<char, Vec<bool>>
}

impl HuffmanBinaryMap {
    pub fn build(huffman_tree: RefHuffmanTree) -> Self {
        let mut map = HashMap::new();
        Self::tree_dfs(&Some(huffman_tree), &mut map, &mut vec![]);
        Self { inner: map }
    }
    fn tree_dfs(
        tree: &Option<RefHuffmanTree>, 
        map: &mut HashMap<char, Vec<bool>>,
        vec: &mut Vec<bool>
    ) {
        if let Some(tree) = tree {
            let tree = tree.as_ref().borrow();
            if let Some(ch) = tree.value {
                map.insert(ch, vec.clone());
            }
            vec.push(false);
            Self::tree_dfs(&tree.left, map, vec);
            let last = vec.last_mut().unwrap();
            *last = true;
            Self::tree_dfs(&tree.right, map, vec);
            vec.pop();
        }
    }
}

/// 用于写入配置文件
impl Display for HuffmanBinaryMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        self.inner.iter()
            .for_each(|(c, vec)| {
                let mut bit_str = String::new();
                vec.iter().for_each(|b| {
                    bit_str += if *b { "1" } else { "0" }
                });
                buf += format!("{}:{}\n", c, bit_str).as_str();
            });
        f.write_str(buf.as_str())
    }
}

pub struct HuffmanCodec;

impl HuffmanCodec {
    /// 哈夫曼编码
    pub fn encode(source: &String) -> (Vec<u8>, String) {
        // 构建字符权重映射
        let weight_map = CharWeightMap::build(&source);
        // 构建哈夫曼树
        let tree = HuffmanTree::build(weight_map);
        // 哈夫曼二进制映射表
        let bit_map = HuffmanBinaryMap::build(tree);
        // println!("{}", bit_map);
        let mut result: Vec<u8> = vec![];
        let (mut buf, mut count) = (0, 0);
        for (_, ch) in source.char_indices() {
            let vec = bit_map.inner.get(&ch).unwrap();
            vec.iter().for_each(|b| {
                buf <<= 1;
                if *b { buf |= 1 }
                count += 1;
                if count >= 8 {
                    result.push(buf);
                    buf = 0;
                    count = 0;
                }
            })
        }
        // 末尾补位数量
        let mut space = 0u8;
        if count != 0 {
            space = 8 - count;
            buf <<= space;
            result.push(buf);
        }
        // 返回的结果
        (
            result, // 压缩后的字节数组
            format!("space:{}\n{}", space, bit_map), // 配置文件内容
        )
    }

    pub fn decode(source: &[u8], decode_map: &DecodeConfig) -> String {
        let mut result = String::new();
        let bit_str = source.iter()
            .map(|num| {
                format!("{u8:>0width$b}", u8=num, width=8)
            })
            .collect::<Vec<String>>()
            .join("");
        // println!("二进制序列：{}", bit_str);

        let mut tmp_str = String::new();
        let last_idx = bit_str.len() - decode_map.space as usize;
        for (i, ch) in bit_str.char_indices() {
            if i >= last_idx {
                break;
            }
            tmp_str.push(ch);
            if let Some(mch) = decode_map.get(&tmp_str) {
                result.push(*mch);
                tmp_str.clear();
            }
        }
        result
    }
}

/// 配置文件的配置
pub struct DecodeConfig {
    pub inner: HashMap<String, char>,
    pub space: u8,
}
impl DecodeConfig {
    pub fn build(source: &String) -> Self {
        let mut map = HashMap::new();
        let mut space = 0u8;
        let arr = source.split("\n");
        for s in arr {
            let pair: Vec<&str> = s.split(":").collect();
            if pair.len() != 2 { 
                continue;
            }
            let (mut ch, bit) = (pair[0], pair[1]);
            match ch {
                "" => ch = "\n",
                "space" => space = u8::from_str_radix(bit, 10).unwrap(),
                _ => (),
            }
            map.insert(bit.to_owned(), ch.chars().nth(0).unwrap());
        };
        Self { inner: map, space }
    }
    pub fn get(&self, k: &String) -> Option<&char> {
        self.inner.get(k)
    }
}
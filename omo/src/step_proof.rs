use crate::emulator::StateChange;
use hex_literal::hex;
use keccak_hasher::KeccakHasher;
use rlp::{DecoderError, Prototype, Rlp, RlpStream};
use std::{borrow::Borrow, collections::BTreeMap, ops::Range};
use trie_db::{
    nibble_ops::NIBBLE_LENGTH,
    node::{NibbleSlicePlan, NodeHandlePlan, NodePlan, Value, ValuePlan},
    ChildReference, NibbleSlice, NodeCodec, TrieMut,
};

const REG_START_ADDR: u64 = 0xffffffff + 1;
pub struct StepProof {}
pub fn generate_step_proof(change: StateChange) -> StepProof {
    let StateChange {
        state_before,
        state_after,
        step,
        access,
    } = change;
    let mem: BTreeMap<_, _> = state_before.memories.into();
    let regs = state_before.regs;

    {
        let mut db = memory_db::MemoryDB::<KeccakHasher, memory_db::HashKey<KeccakHasher>, _>::new(
            RlpNodeCodec::empty_node(),
        );
        let mut trie = trie_db::TrieDBMutBuilder::<EthTrieLayout>::new(&mut db, &mut root).build();
        for (addr, v) in mem {
            trie.insert(&addr.to_be_bytes(), v.as_slice()).unwrap();
        }
        for (reg_id, v) in regs {
            trie.insert(
                &(REG_START_ADDR + (reg_id as u64) * 4).to_be_bytes(),
                &v.to_be_bytes(),
            )
            .unwrap();
        }
        trie.commit()
    }

    StepProof {}
}

#[test]
fn test_trie_root() {
    use memory_db::HashKey;
    let mut db = memory_db::MemoryDB::<KeccakHasher, HashKey<KeccakHasher>, _>::new(&[0x80]);

    let mut root = Default::default();
    let mut trie = trie_db::TrieDBMutBuilder::<EthTrieLayout>::new(&mut db, &mut root).build();

    {
        trie.insert(b"foo", b"foo").unwrap();
        assert_eq!(
            *trie.root(),
            hex!("51d8ccee4184b078b508033281a3dc892194afc17b3e92ae7e4a5b400e8454cc")
        );
    }
    {
        trie.insert(b"fooo", b"fooo").unwrap();
        assert_eq!(
            *trie.root(),
            hex!("a6a751b890341768940a99f4e6b337a3c279e014fc0980a4d96ec72225567add")
        );
    }
    {
        trie.insert(b"foa", b"foa").unwrap();
        assert_eq!(
            *trie.root(),
            hex!("227cd158eb4ad8a5169fdbd13c7d906ccf28937d21cea2fa7635e941c7c5cc65")
        );
    }
    {
        trie.insert(b"fooa", b"fooa").unwrap();
        assert_eq!(
            *trie.root(),
            hex!("87b08ece907edf5c5c19e56beb0ed9badf7bbec61f5e686ca0e31a220e0d4b19")
        );
    }
    {
        trie.insert(b"fooa", b"foob").unwrap();
        assert_eq!(
            *trie.root(),
            hex!("55f7f9d2d7117ebefcfc94b0c3b526508ecff533e8f1b0405ff22f6f5c73ebd2")
        );
    }
}

#[derive(Default, Clone)]
pub struct EthTrieLayout;

impl trie_db::TrieLayout for EthTrieLayout {
    const USE_EXTENSION: bool = true;
    const ALLOW_EMPTY: bool = false;
    const MAX_INLINE_VALUE: Option<u32> = None;
    type Hash = KeccakHasher;
    type Codec = RlpNodeCodec;
}

impl trie_db::TrieConfiguration for EthTrieLayout {}

const HASHED_NULL_NODE_BYTES: [u8; 32] = [
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];
//const HASHED_NULL_NODE: H256 = H256(HASHED_NULL_NODE_BYTES);

pub struct RlpNodeCodec;

impl NodeCodec for RlpNodeCodec {
    type Error = DecoderError;
    type HashOut = [u8; 32];

    fn hashed_null_node() -> Self::HashOut {
        HASHED_NULL_NODE_BYTES
    }

    fn decode_plan(data: &[u8]) -> Result<NodePlan, Self::Error> {
        let r = Rlp::new(data);
        match r.prototype()? {
            // empty branch index
            Prototype::Data(0) => Ok(NodePlan::Empty),
            // leaf of extension
            Prototype::List(2) => {
                let (first_elem, offset) = r.at_with_offset(0)?;
                let first_elem_payload_info = first_elem.payload_info()?;
                let first_elem_data = first_elem.data()?;
                let partial = NibbleSlicePlan::new(
                    Range {
                        start: offset + first_elem_payload_info.header_len,
                        end: offset + first_elem_payload_info.total(),
                    },
                    if first_elem_data[0] & 16 == 16 { 1 } else { 2 },
                );
                let (value_elem, offset) = r.at_with_offset(1)?;
                if first_elem_data[0] & 32 == 32 {
                    let value_elem_payload_info = value_elem.payload_info()?;
                    Ok(NodePlan::Leaf {
                        partial,
                        value: ValuePlan::Inline(
                            (offset + value_elem_payload_info.header_len)
                                ..(offset + value_elem_payload_info.total()),
                        ),
                    })
                } else {
                    let child = if value_elem.as_raw().len() >= 32 {
                        let payload_info = value_elem.payload_info()?;
                        NodeHandlePlan::Hash(
                            (offset + payload_info.header_len)..(offset + payload_info.total()),
                        )
                    } else {
                        NodeHandlePlan::Inline(offset..(offset + value_elem.as_raw().len()))
                    };
                    Ok(NodePlan::Extension { partial, child })
                }
            }
            // branch
            Prototype::List(17) => {
                let mut children = [
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                    None, None, None,
                ];
                for i in 0..16 {
                    let (elem, offset) = r.at_with_offset(i)?;
                    if elem.is_empty() {
                        children[i] = None;
                    } else {
                        children[i] = Some(if elem.as_raw().len() >= 32 {
                            let payload_info = elem.payload_info()?;
                            NodeHandlePlan::Hash(
                                (offset + payload_info.header_len)..(offset + payload_info.total()),
                            )
                        } else {
                            NodeHandlePlan::Inline(offset..(offset + elem.as_raw().len()))
                        });
                    }
                }
                let mut value = {
                    let (elem, offset) = r.at_with_offset(16)?;
                    if elem.is_empty() {
                        None
                    } else {
                        let elem_payload_info = elem.payload_info()?;
                        Some(ValuePlan::Inline(
                            (offset + elem_payload_info.header_len)
                                ..(offset + elem_payload_info.total()),
                        ))
                    }
                };
                Ok(NodePlan::Branch { children, value })
            }
            _ => Err(DecoderError::Custom("Rlp is not valid.")),
        }
    }

    fn is_empty_node(data: &[u8]) -> bool {
        Rlp::new(data).is_empty()
    }

    fn empty_node() -> &'static [u8] {
        &[0x80]
    }

    fn leaf_node(
        mut partial: impl Iterator<Item = u8>,
        number_nibble: usize,
        value: Value,
    ) -> Vec<u8> {
        let offset = (number_nibble % 2) as u8;
        let prefix = (offset + 2) << 4;

        let mut stream = RlpStream::new_list(2);
        if offset == 0 {
            stream.append_iter(vec![prefix].into_iter().chain(partial));
        } else {
            stream.append_iter(
                vec![prefix + partial.next().unwrap()]
                    .into_iter()
                    .chain(partial),
            );
        }
        match value {
            Value::Inline(v) => {
                stream.append(&v);
            }
            Value::Node(_) => {
                unimplemented!()
            }
        }
        stream.out().to_vec()
    }

    fn extension_node(
        mut partial: impl Iterator<Item = u8>,
        number_nibble: usize,
        child_ref: ChildReference<Self::HashOut>,
    ) -> Vec<u8> {
        let offset = (number_nibble % 2) as u8;
        let prefix = offset << 4;

        let mut stream = RlpStream::new_list(2);

        if offset == 0 {
            stream.append_iter(vec![prefix].into_iter().chain(partial));
        } else {
            stream.append_iter(
                vec![prefix + partial.next().unwrap()]
                    .into_iter()
                    .chain(partial),
            );
        }
        match child_ref {
            ChildReference::Hash(h) => {
                stream.append_iter(h);
            }
            ChildReference::Inline(h, size) => {
                stream.append_raw(&h[..size], 1);
            }
        }
        stream.out().to_vec()
    }

    fn branch_node(
        children: impl Iterator<Item = impl Borrow<Option<ChildReference<Self::HashOut>>>>,
        value: Option<Value>,
    ) -> Vec<u8> {
        let mut stream = RlpStream::new_list(17);
        for child_ref in children {
            match child_ref.borrow() {
                Some(c) => match c {
                    ChildReference::Hash(h) => stream.append(&h.as_slice()),
                    ChildReference::Inline(h, size) => stream.append_raw(&h[..*size], 1),
                },
                None => stream.append_empty_data(),
            };
        }
        if let Some(v) = value {
            match v {
                Value::Inline(v) => stream.append(&v),
                Value::Node(_) => unimplemented!(),
            };
        } else {
            stream.append_empty_data();
        }
        stream.out().to_vec()
    }

    fn branch_node_nibbled(
        _partial: impl Iterator<Item = u8>,
        _number_nibble: usize,
        _children: impl Iterator<Item = impl Borrow<Option<ChildReference<Self::HashOut>>>>,
        _value: Option<Value>,
    ) -> Vec<u8> {
        unreachable!("codec with extension branch")
    }
}

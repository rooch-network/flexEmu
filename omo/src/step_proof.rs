use crate::emulator::StateChange;
use ethtrie_codec::{EthTrieLayout, KeccakHasher, RlpNodeCodec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use trie_db::{NodeCodec, TrieMut};

const REG_START_ADDR: u64 = 0xffffffff + 1;
use hex_buffer_serde::{Hex, HexForm};
use unicorn_engine::RegisterMIPS;

#[derive(Serialize, Deserialize, Debug)]
pub struct StepProof {
    #[serde(with = "HexForm")]
    root_before: [u8; 32],
    #[serde(with = "HexForm")]
    root_after: [u8; 32],
    #[serde(with = "HexForm")]
    access_nodes: Vec<u8>,

    #[serde(with = "HexForm")]
    regs_before: Vec<u8>,
    #[serde(with = "HexForm")]
    regs_after: Vec<u8>,
}

pub fn generate_step_proof(change: StateChange) -> StepProof {
    let StateChange {
        state_before,
        state_after,
        step: _,
        access,
    } = change;
    let mut root = Default::default();
    let mut db = memory_db::MemoryDB::<KeccakHasher, memory_db::HashKey<KeccakHasher>, _>::new(
        RlpNodeCodec::empty_node(),
    );

    {
        let mem: BTreeMap<_, _> = state_before.memories.into();

        let mut trie = trie_db::TrieDBMutBuilder::<EthTrieLayout>::new(&mut db, &mut root).build();
        for (addr, v) in mem {
            let shortend_addr = (addr >> 2) as u32;
            trie.insert(&shortend_addr.to_be_bytes(), v.as_slice())
                .unwrap();
        }
        // FIXME: ignore register for now, as there is no way to get the reg access when executing a single step.
        // for (reg_id, v) in state_before.regs.clone() {
        //     let addr = ((REG_START_ADDR + (reg_id as u64) * 4) >> 2) as u32;
        //     trie.insert(&addr.to_be_bytes(), &(v as u32).to_be_bytes())
        //         .unwrap();
        // }

        trie.commit();

        // for (reg_id, v) in state_before.regs {
        //     let addr = ((REG_START_ADDR + (reg_id as u64) * 4) >> 2) as u32;
        //     let read_back = trie.get(&addr.to_be_bytes()).unwrap().unwrap();
        //     let read_back =
        //         u32::from_be_bytes(*read_back.as_slice().as_chunks::<4>().0.first().unwrap());
        //     assert_eq!(v, read_back as u64);
        //     println!("reg {}: {}", reg_id, read_back);
        // }
    };
    let root_before = root;

    let accessed_nodes = {
        let mut recorder = trie_db::recorder::Recorder::<EthTrieLayout>::default();
        let mut trie =
            trie_db::TrieDBMutBuilder::<EthTrieLayout>::from_existing(&mut db, &mut root)
                .with_recorder(&mut recorder)
                .build();
        for acc in &access {
            assert_eq!(acc.addr & 3, 0, "addr {:#x} not 4byte aligned", acc.addr);
            assert_eq!(acc.size, 4, "mem size {} not 4", acc.size);

            // FIXME: if read/or write is not 4bytes, this will be error.
            if acc.write {
                let _old_value = trie
                    .insert(
                        ((acc.addr >> 2) as u32).to_be_bytes().as_slice(),
                        (acc.value as u32).to_be_bytes().as_slice(),
                    )
                    .unwrap();
            } else {
                let read_result = trie
                    .get(((acc.addr >> 2) as u32).to_be_bytes().as_slice())
                    .unwrap();
                assert!(read_result.is_some());
            }
        }
        // for (reg_id, v) in state_after.regs.clone() {
        //     let addr = ((REG_START_ADDR + (reg_id as u64) * 4) >> 2) as u32;
        //     trie.insert(&addr.to_be_bytes(), &(v as u32).to_be_bytes())
        //         .unwrap();
        // }
        // for (reg_id, v) in state_after.regs {
        //     let addr = ((REG_START_ADDR + (reg_id as u64) * 4) >> 2) as u32;
        //     let read_back = trie.get(&addr.to_be_bytes()).unwrap().unwrap();
        //     let read_back =
        //         u32::from_be_bytes(*read_back.as_slice().as_chunks::<4>().0.first().unwrap());
        //     assert_eq!(v, read_back as u64);
        //     println!("after, reg {}: {}", reg_id, read_back);
        // }
        trie.commit();
        drop(trie);
        recorder.drain()
    };
    let mut encoded_nodes = rlp::RlpStream::new_list(accessed_nodes.len());
    for v in accessed_nodes {
        encoded_nodes.append(&v.data);
    }

    let root_after = root;

    // {
    //     let mem: BTreeMap<_, _> = state_after.memories.into();
    //     let mut db = memory_db::MemoryDB::<KeccakHasher, memory_db::HashKey<KeccakHasher>, _>::new(
    //         RlpNodeCodec::empty_node(),
    //     );
    //     let mut trie = trie_db::TrieDBMutBuilder::<EthTrieLayout>::new(&mut db, &mut root).build();
    //     for (addr, v) in mem {
    //         trie.insert(&addr.to_be_bytes(), v.as_slice()).unwrap();
    //     }
    //     assert_eq!(
    //         trie.root(),
    //         &root_after,
    //         "root_after should be equal to trie root"
    //     );
    // }
    let regs_before = {
        let mut encoder = rlp::RlpStream::new_list(state_before.regs.len());
        for (reg_id, v) in state_before.regs.clone() {
            let encoded_register = ((reg_id as u64) << 32) + v;
            encoder.append_iter(encoded_register.to_be_bytes());
        }
        encoder.out().to_vec()
    };
    let regs_after = {
        let mut encoder = rlp::RlpStream::new_list(state_after.regs.len());
        for (reg_id, v) in state_after.regs.clone() {
            let encoded_register = ((reg_id as u64) << 32) + v;
            encoder.append_iter(encoded_register.to_be_bytes());
        }
        encoder.out().to_vec()
    };

    StepProof {
        root_before,
        root_after,
        access_nodes: encoded_nodes.out().to_vec(),
        regs_before,
        regs_after,
    }
}

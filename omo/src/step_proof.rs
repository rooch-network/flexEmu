use crate::emulator::StateChange;
use ethtrie_codec::{EthTrieLayout, KeccakHasher, RlpNodeCodec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use trie_db::{NodeCodec, TrieMut};

const REG_START_ADDR: u64 = 0xffffffff + 1;

#[derive(Serialize, Deserialize, Debug)]
pub struct StepProof {
    root_before: [u8; 32],
    root_after: [u8; 32],
    access_nodes: Vec<Vec<u8>>,
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
        {
            let mut trie =
                trie_db::TrieDBMutBuilder::<EthTrieLayout>::new(&mut db, &mut root).build();
            for (addr, v) in mem {
                trie.insert(&addr.to_be_bytes(), v.as_slice()).unwrap();
            }
            // FIXME: ignore register for now, as there is no way to get the reg access when executing a single step.
            for (reg_id, v) in state_before.regs {
                trie.insert(
                    &(REG_START_ADDR + (reg_id as u64) * 4).to_be_bytes(),
                    &v.to_be_bytes(),
                )
                .unwrap();
            }
            trie.commit();
        }
    }
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
                        acc.addr.to_be_bytes().as_slice(),
                        (acc.value as u64).to_be_bytes().as_slice(),
                    )
                    .unwrap();
            } else {
                let read_result = trie.get(acc.addr.to_be_bytes().as_slice()).unwrap();
                assert!(read_result.is_some());
            }
        }
        for (reg_id, v) in state_after.regs {
            trie.insert(
                &(REG_START_ADDR + (reg_id as u64) * 4).to_be_bytes(),
                &v.to_be_bytes(),
            )
            .unwrap();
        }
        trie.commit();
        drop(trie);
        recorder.drain()
    };

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
    StepProof {
        root_before,
        root_after,
        access_nodes: accessed_nodes.into_iter().map(|v| v.data).collect(),
    }
}


module omo::Challenge {
    use StarcoinFramework::Table;
    use StarcoinFramework::Hash;
    use StarcoinFramework::Vector;
    use SFC::RLP;
    use StarcoinFramework::Signer;
    use omo::memory;
    use omo::mips_emulator;
    use ContractAddress::ContractAccount;
    use trie::hash_value::HashValue;
    use trie::hash_value;
    use StarcoinFramework::Event::emit_event;

    const ERR_BLOCK_NUMBER_HASH_EMPTY: u64 = 1;
    const ERR_BLOCK_NUMBER_P1_HASH_EMPTY: u64 = 2;
    const ERR_PARENT_BLOCK_HASH_WRONG: u64 = 3;
    const ERR_NOT_CHALLENGE: u64 = 4;
    const ERR_MIPS_MACHINE_NOT_STOPPED: u64 = 5;
    const ERR_FINAL_STATE_ROOT_NOT_WRITTEN: u64 = 6;
    const ERR_MIPS_MACHINE_DIFFERENT_STATE_ROOT: u64 = 7;
    const ERR_INVALID_CHALLENGE: u64 = 8;
    const ERR_MUST_BE_CHALLENGER: u64 = 9;
    const ERR_MUST_BE_SEARCHING: u64 = 10;
    const ERR_STATE_ALREADY_PROPOSED: u64 = 11;
    const ERR_CHALLENGE_STATE_NOT_PROPOSED: u64 = 12;
    const ERR_BINARY_SEARCH_NOT_FINISHED: u64 = 13;
    const ERR_WRONG_ASSERTED_STATE_FOR_CHALLENGER: u64 = 14;
    const ERR_WRONG_ASSERTED_STATE_FOR_DEFENDER: u64 = 15;

    struct ChallengeData has key,store {
        L: u64,
        R: u64,
        asserted_state: Table::Table<u64, HashValue>,
        defended_state: Table::Table<u64, HashValue>,
        challenger: address,
        block_number_n_hash: HashValue,
    }

    struct Challenges has key,store {
        value: vector<ChallengeData>
    }

    struct Global has key,store {
        globalStartState: HashValue,
        lastChallengeId: u64,
    }

    fun vector_to_u64(v: &vector<u8>): u64 {
        if (Vector::length(v) < 8) {
            return 0
        };
        let r: u64 = 0;
        r = r + ((*Vector::borrow(v, 7) << 0) as u64);
        r = r + ((*Vector::borrow(v, 6) << 8) as u64);
        r = r + ((*Vector::borrow(v, 5) << 16) as u64);
        r = r + ((*Vector::borrow(v, 4) << 24) as u64);
        r = r + ((*Vector::borrow(v, 3) << 32) as u64);
        r = r + ((*Vector::borrow(v, 2) << 40) as u64);
        r = r + ((*Vector::borrow(v, 1) << 48) as u64);
        r = r + ((*Vector::borrow(v, 0) << 56) as u64);
        r
    }

    public entry fun do_init(account: signer, start_state: HashValue) {
        Self::init(&account, start_state);
    }

    public fun init(signer: &signer, global_start_sate: HashValue) {
        let g = Global {
            globalStartState: global_start_sate,
            lastChallengeId: 0,
        };
        move_to(signer, g);

        let challenges = Challenges{value: Vector::empty()};
        move_to(signer, challenges);

        // initial memory
        memory::create(signer);
    }

    fun get_gloal_start_state(signer: &signer): HashValue acquires Global {
        let g = borrow_global<Global>(Signer::address_of(signer));
        *&g.globalStartState
    }

    fun get_global_challenge_id(signer: &signer): u64 acquires Global {
        let g = borrow_global<Global>(Signer::address_of(signer));
        *&g.lastChallengeId
    }

    fun set_challenge_challenger(signer: &signer, change_id: u64, challenger: address,
                                 block_n_hash: HashValue, asserted_state: HashValue, defended_state: HashValue,
                                 final_step_count: u64, final_system_state: HashValue, l: u64, r: u64)
    acquires Challenges {
        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, change_id);

        c.challenger = challenger;
        c.block_number_n_hash = block_n_hash;

        Table::add(&mut c.asserted_state, 0, asserted_state);
        Table::add(&mut c.defended_state, 0, defended_state);

        Table::add(&mut c.defended_state, final_step_count, final_system_state);

        c.L = l;
        c.R = r;
    }

    public fun initiateChallenge(signer: &signer, blockNumberN_hash: HashValue, blockNumberN_P_1_hash: HashValue,
                                 blockHeader_N_P1: HashValue, assertionRoots: HashValue,
                                 finalSystemState: HashValue, step_count: u64): u64 acquires Global, Challenges {
        if (Vector::length(hash_value::bytes(&blockNumberN_hash)) == 0) {
            abort ERR_BLOCK_NUMBER_HASH_EMPTY
        };

        if (Vector::length(hash_value::bytes(&blockNumberN_P_1_hash)) == 0) {
            abort ERR_BLOCK_NUMBER_P1_HASH_EMPTY
        };

        let input_hash: vector<u8>;
        {
            let decode_headder = RLP::decode_list(hash_value::bytes(&blockHeader_N_P1));
            let parent_hash = Vector::borrow(&decode_headder, 0);
            assert!(hash_value::new(*parent_hash) == blockNumberN_hash, ERR_PARENT_BLOCK_HASH_WRONG);

            let newroot = Vector::borrow(&decode_headder, 3);
            assert!(hash_value::new(*newroot) != assertionRoots, ERR_NOT_CHALLENGE);

            let txhash = Vector::borrow(&decode_headder, 4);
            let coinbase = Vector::borrow(&decode_headder, 2);
            let gaslimit = Vector::borrow(&decode_headder, 1);
            let time = Vector::borrow(&decode_headder, 9);

            let input_hash_vector: vector<u8> = Vector::empty();
            Vector::append(&mut input_hash_vector, *parent_hash);
            Vector::append(&mut input_hash_vector, *txhash);
            Vector::append(&mut input_hash_vector, *coinbase);
            Vector::append(&mut input_hash_vector, *gaslimit);
            Vector::append(&mut input_hash_vector, *time);
            input_hash = Hash::keccak_256(input_hash_vector);
        };

        let mem = memory::get_mem(Signer::address_of(&ContractAccount::get_contract_signer()));

        let start_state = get_gloal_start_state(signer);
        start_state = memory::write_memory(&mut mem, start_state, 0x30000000, vector_to_u64(&input_hash));

        let p1 = memory::read_memory(&mut mem, finalSystemState, 0xC0000000);
        assert!(p1 == 0x5EAD0000, ERR_MIPS_MACHINE_NOT_STOPPED);

        let p2 = memory::read_memory(&mut mem, finalSystemState, 0x30000800);
        assert!(p2 == 0x1337F00d, ERR_FINAL_STATE_ROOT_NOT_WRITTEN);

        assert!(memory::read_memory(&mut mem, finalSystemState, 0x30000804) == vector_to_u64(hash_value::bytes(&assertionRoots)),
            ERR_MIPS_MACHINE_DIFFERENT_STATE_ROOT);

        let challenge_id = get_global_challenge_id(signer);

        set_challenge_challenger(signer, challenge_id,
            Signer::address_of(signer),
            blockNumberN_hash,
            copy start_state,
            copy start_state,
            step_count,
            finalSystemState,
            0,
            step_count
        );

        memory::return_mem(mem);

        0
    }

    public fun is_searching(challenge_id: u64): bool acquires Challenges {
        let contract_signer = ContractAccount::get_contract_signer();
        let challenges = borrow_global_mut<Challenges>(Signer::address_of(&contract_signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        (c.L + 1 != c.R)
    }

    public fun get_step_number(challenge_id: u64): u64 acquires Challenges {
        let contract_signer = ContractAccount::get_contract_signer();
        let challenges = borrow_global_mut<Challenges>(Signer::address_of(&contract_signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        (c.L + c.R) / 2
    }

    public fun get_proposed_state(challenge_id: u64): HashValue acquires Challenges {
        let step_number = get_step_number(challenge_id);

        let contract_signer = ContractAccount::get_contract_signer();
        let challenges = borrow_global_mut<Challenges>(Signer::address_of(&contract_signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        let borrowed_state = Table::borrow(&c.asserted_state, step_number);
        *borrowed_state
    }

    // the challenger call this function to submit the state hash
    // for next step in the binary search
    public fun propose_state(signer: &signer, challenge_id: u64, state_hash: HashValue) acquires Challenges {
        assert!(is_searching(challenge_id), ERR_MUST_BE_SEARCHING);
        let step_number = get_step_number(challenge_id);

        let contract_signer = ContractAccount::get_contract_signer();
        let challenges = borrow_global_mut<Challenges>(Signer::address_of(&contract_signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        assert!(c.challenger == Signer::address_of(signer), ERR_MUST_BE_CHALLENGER);

        let state = Table::borrow(&c.asserted_state, step_number);
        assert!(vector_to_u64(&hash_value::to_bytes(*state)) == 0, ERR_STATE_ALREADY_PROPOSED);

        let key_contains = Table::contains(&c.asserted_state, step_number);
        if (!key_contains) {
            Table::add(&mut c.asserted_state, step_number, state_hash);
        }
    }

    // the defender call this function to submit the state hash
    // for next step in the binary search
    public fun respond_state(signer: &signer, challenge_id: u64, state_hash: HashValue) acquires Challenges {
        assert!(is_searching(challenge_id), ERR_MUST_BE_SEARCHING);
        let step_number = get_step_number(challenge_id);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        assert!(c.challenger == Signer::address_of(signer), ERR_MUST_BE_CHALLENGER);

        let asserted_state = Table::borrow(&c.asserted_state, step_number);
        assert!(vector_to_u64(&hash_value::to_bytes(*asserted_state)) != 0, ERR_CHALLENGE_STATE_NOT_PROPOSED);

        let defended_state = Table::borrow(&c.asserted_state, step_number);
        assert!(vector_to_u64(&hash_value::to_bytes(*defended_state)) != 0, ERR_STATE_ALREADY_PROPOSED);

        let key_contains = Table::contains(&c.defended_state, step_number);
        if (!key_contains) {
            Table::add(&mut c.defended_state, step_number, state_hash);
        };

        // update binary search bounds
        let asserted_state = Table::borrow(&c.asserted_state, step_number);
        let defended_state = Table::borrow(&c.defended_state, step_number);
        if (asserted_state == defended_state) {
            c.L = step_number;  // agree
        } else {
            c.R = step_number;  // disagree
        }
    }

    public fun confirm_state_transition(signer: &signer, challenge_id: u64) acquires Challenges {
        assert!(!is_searching(challenge_id), ERR_BINARY_SEARCH_NOT_FINISHED);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        let asserted_state = Table::borrow(&c.asserted_state, c.L);
        let addr = Signer::address_of(signer);
        let step_state = mips_emulator::run(addr, hash_value::to_bytes(*asserted_state));

        let right_asserted_state = Table::borrow(&c.asserted_state, c.R);
        assert!(step_state == hash_value::to_bytes(*right_asserted_state), ERR_WRONG_ASSERTED_STATE_FOR_CHALLENGER);

        // TODO: emit challenge wins event
    }

    public fun deny_state_transition(signer: &signer, challenge_id: u64) acquires Challenges {
        assert!(!is_searching(challenge_id), ERR_BINARY_SEARCH_NOT_FINISHED);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        let defended_state = Table::borrow(&c.defended_state, c.L);
        let addr = Signer::address_of(signer);
        let step_state = mips_emulator::run(addr, hash_value::to_bytes(*defended_state));

        let right_defended_state = Table::borrow(&c.defended_state, c.R);
        assert!(step_state == hash_value::to_bytes(*right_defended_state), ERR_WRONG_ASSERTED_STATE_FOR_DEFENDER);

        // TODO: emit defender wins event
    }
}



module omo::SimpleChallenge {
    use StarcoinFramework::Table;
    use StarcoinFramework::Vector;
    use StarcoinFramework::Signer;
    use omo::mips_emulator;
    use trie::hash_value::HashValue;
    use trie::hash_value;
    use StarcoinFramework::Signer::address_of;

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
    const ERR_TABLE_KEY_NOT_EXISTS: u64 = 16;

    struct ChallengeData has key,store {
        L: u64,
        R: u64,
        asserted_state: Table::Table<u64, HashValue>,
        defended_state: Table::Table<u64, HashValue>,
        challenger: address,
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

    fun borrow_table_value(table: &Table::Table<u64, HashValue>, key: u64): &HashValue {
        assert!(Table::contains(table, key), ERR_TABLE_KEY_NOT_EXISTS);
        borrow_table_value(table, key)
    }

    public(script) fun init(signer: &signer, global_start_sate: HashValue) {
        let g = Global {
            globalStartState: global_start_sate,
            lastChallengeId: 0,
        };
        move_to(signer, g);

        let challenges = Challenges{value: Vector::empty<ChallengeData>()};
        move_to(signer, challenges);
    }

    fun get_global_start_state(signer: &signer): HashValue acquires Global {
        let g = borrow_global<Global>(Signer::address_of(signer));
        *&g.globalStartState
    }

    fun get_global_challenge_id(signer: &signer): u64 acquires Global {
        let g = borrow_global_mut<Global>(Signer::address_of(signer));
        g.lastChallengeId = g.lastChallengeId + 1;
        *&g.lastChallengeId
    }

    fun set_challenge(signer: &signer, challenge_id: u64, challenger: address,
                                 asserted_state: HashValue, defended_state: HashValue,
                                 final_step_count: u64, final_system_state: HashValue, l: u64, r: u64)
    acquires Challenges {
        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));

        if (challenge_id > Vector::length(&challenges.value)) {
            let challenge_data = ChallengeData{
                L: 0,
                R: 0,
                asserted_state: Table::new(),
                defended_state: Table::new(),
                challenger: @0x0,
            };
            Vector::push_back(&mut challenges.value, challenge_data);
        };

        let c = Vector::borrow_mut<ChallengeData>(&mut challenges.value, 0);

        c.challenger = challenger;

        Table::add(&mut c.asserted_state, 0, asserted_state);
        Table::add(&mut c.defended_state, 0, defended_state);

        Table::add(&mut c.defended_state, final_step_count, final_system_state);

        c.L = l;
        c.R = r;
    }

    public(script) fun initiateChallenge(signer: &signer, finalSystemState: HashValue, step_count: u64) acquires Global, Challenges {
        let start_state = get_global_start_state(signer);

        let challenge_id = get_global_challenge_id(signer);

        set_challenge(signer, challenge_id,
            Signer::address_of(signer),
            copy start_state,
            copy start_state,
            step_count,
            finalSystemState,
            0,
            step_count
        );
    }

    public(script) fun is_searching(address: address, challenge_id: u64): bool acquires Challenges {
        let challenges = borrow_global_mut<Challenges>(address);
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        (c.L + 1 != c.R)
    }

    fun get_step_number(address: address, challenge_id: u64): u64 acquires Challenges {
        let challenges = borrow_global_mut<Challenges>(address);
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        (c.L + c.R) / 2
    }

    public(script) fun get_proposed_state(signer: &signer, challenge_id: u64): HashValue acquires Challenges {
        let step_number = get_step_number(address_of(signer), challenge_id);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        let borrowed_state = borrow_table_value(&c.asserted_state, step_number);
        *borrowed_state
    }

    // the challenger call this function to submit the state hash
    // for next step in the binary search
    public(script) fun propose_state(signer: &signer, challenge_id: u64, state_hash: HashValue) acquires Challenges {
        assert!(is_searching(address_of(signer), challenge_id), ERR_MUST_BE_SEARCHING);
        let step_number = get_step_number(address_of(signer), challenge_id);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        assert!(c.challenger == Signer::address_of(signer), ERR_MUST_BE_CHALLENGER);

        let state = borrow_table_value(&c.asserted_state, step_number);
        assert!(vector_to_u64(&hash_value::to_bytes(*state)) == 0, ERR_STATE_ALREADY_PROPOSED);

        let key_contains = Table::contains(&c.asserted_state, step_number);
        if (!key_contains) {
            Table::add(&mut c.asserted_state, step_number, state_hash);
        }
    }

    // the defender call this function to submit the state hash
    // for next step in the binary search
    public(script) fun respond_state(signer: &signer, challenge_id: u64, state_hash: HashValue) acquires Challenges {
        assert!(is_searching(address_of(signer), challenge_id), ERR_MUST_BE_SEARCHING);
        let step_number = get_step_number(address_of(signer), challenge_id);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        assert!(c.challenger == Signer::address_of(signer), ERR_MUST_BE_CHALLENGER);

        let asserted_state = borrow_table_value(&c.asserted_state, step_number);
        assert!(vector_to_u64(&hash_value::to_bytes(*asserted_state)) != 0, ERR_CHALLENGE_STATE_NOT_PROPOSED);

        let defended_state = borrow_table_value(&c.asserted_state, step_number);
        assert!(vector_to_u64(&hash_value::to_bytes(*defended_state)) != 0, ERR_STATE_ALREADY_PROPOSED);

        let key_contains = Table::contains(&c.defended_state, step_number);
        if (!key_contains) {
            Table::add(&mut c.defended_state, step_number, state_hash);
        };

        // update binary search bounds
        let asserted_state = borrow_table_value(&c.asserted_state, step_number);
        let defended_state = borrow_table_value(&c.defended_state, step_number);
        if (asserted_state == defended_state) {
            c.L = step_number;  // agree
        } else {
            c.R = step_number;  // disagree
        }
    }

    public(script) fun confirm_state_transition(signer: &signer, challenge_id: u64) acquires Challenges {
        assert!(!is_searching(address_of(signer), challenge_id), ERR_BINARY_SEARCH_NOT_FINISHED);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        let asserted_state = borrow_table_value(&c.asserted_state, c.L);
        let addr = Signer::address_of(signer);
        let step_state = mips_emulator::run(addr, hash_value::to_bytes(*asserted_state));

        let right_asserted_state = borrow_table_value(&c.asserted_state, c.R);
        assert!(step_state == hash_value::to_bytes(*right_asserted_state), ERR_WRONG_ASSERTED_STATE_FOR_CHALLENGER);

        // TODO: emit challenge wins event
    }

    public(script) fun deny_state_transition(signer: &signer, challenge_id: u64) acquires Challenges {
        // if c.L + 1 == c.R, run the following code
        assert!(!is_searching(address_of(signer), challenge_id), ERR_BINARY_SEARCH_NOT_FINISHED);

        let challenges = borrow_global_mut<Challenges>(Signer::address_of(signer));
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        let defended_state = borrow_table_value(&c.defended_state, c.L);
        let addr = Signer::address_of(signer);
        let step_state = mips_emulator::run(addr, hash_value::to_bytes(*defended_state));

        let right_defended_state = borrow_table_value(&c.defended_state, c.R);
        assert!(step_state == hash_value::to_bytes(*right_defended_state), ERR_WRONG_ASSERTED_STATE_FOR_DEFENDER);

        // TODO: emit defender wins event
    }
}


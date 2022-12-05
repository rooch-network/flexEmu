
module omo::SimpleChallenge {
    use StarcoinFramework::Table;
    use StarcoinFramework::Vector;
    use StarcoinFramework::Signer;
    use omo::mips_emulator;
    use trie::hash_value::HashValue;
    use trie::hash_value;

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
    const ERR_STATE_ARE_SAME: u64 = 17;

    struct ChallengeData has key,store {
        l: u64,
        r: u64,
        asserted_state: Table::Table<u64, HashValue>,
        defended_state: Table::Table<u64, HashValue>,
        challenger: address,
    }

    struct Challenges has key,store {
        value: vector<ChallengeData>
    }

    struct Global has key,store {
        // last_challenge_d: u64,
        declared_state: HashValue,
        // step_count: u64,
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

    // fun next_challenge_id(proposer: address): u64 acquires Global {
    //     let g = borrow_global_mut<Global>(proposer);
    //     g.last_challenge_d = g.last_challenge_d + 1;
    //     *&g.last_challenge_d
    // }

    public fun declare_state(signer: &signer, final_state: HashValue) {
        let g = Global {
            declared_state: final_state,
        };
        move_to(signer, g);
        let challenges = Challenges{value: Vector::empty<ChallengeData>()};
        move_to(signer, challenges);
    }

    public fun create_challenge(signer: &signer, proposer_address: address, final_system_state: HashValue, step_count: u64): u64
    acquires Global, Challenges {
        //let challenge_id = next_challenge_id(proposer_address);
        let glo = borrow_global<Global>(proposer_address);
        assert!(glo.declared_state != final_system_state, ERR_STATE_ARE_SAME);

        let challenges = borrow_global_mut<Challenges>(proposer_address);
        let challenge_data = ChallengeData {
            l: 0,
            r: step_count,
            asserted_state: Table::new(),
            defended_state: Table::new(),
            challenger: Signer::address_of(signer),
        };
        Table::add(&mut challenge_data.defended_state, step_count, final_system_state);

        Vector::push_back(&mut challenges.value, challenge_data);
        Vector::length(&challenges.value) - 1
    }

    public fun is_searching(address: address, challenge_id: u64): bool acquires Challenges {
        let challenges = borrow_global<Challenges>(address);
        let c = Vector::borrow(&challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        is_searching_(c)
    }

    /// Check whether a participant has already propose state of some step
    public fun contain_state(proposer_address: address, challenge_id: u64, step: u64, defend: bool): bool acquires Challenges {
        let challenges = borrow_global<Challenges>(proposer_address);
        let c = Vector::borrow(&challenges.value, challenge_id);
        if (defend) {
            Table::contains(&c.defended_state, step)
        } else {
            Table::contains(&c.asserted_state, step)
        }
    }

    fun get_step_number(address: address, challenge_id: u64): u64 acquires Challenges {
        let challenges = borrow_global<Challenges>(address);
        let c = Vector::borrow(&challenges.value, challenge_id);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);

        step_number(c)
    }

    fun is_searching_(c: &ChallengeData): bool {
        (c.l + 1 != c.r)
    }
    fun step_number(c: &ChallengeData): u64 {
        (c.l + c.r) / 2
    }

    public fun get_proposed_state(user: address, challenge_id: u64): HashValue acquires Challenges {
        let challenges = borrow_global<Challenges>(user);
        let c = Vector::borrow(&challenges.value, challenge_id);
        let step_number = step_number(c);

        assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        let borrowed_state = Table::borrow(&c.asserted_state, step_number);
        *borrowed_state
    }

    // the challenger call this function to submit the state hash
    // for next step in the binary search
    public fun assert_state(signer: &signer, proposer_address: address, challenge_id: u64, state_hash: HashValue) acquires Challenges {
        let challenges = borrow_global_mut<Challenges>(proposer_address);
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);
        assert!(is_searching_(c), ERR_MUST_BE_SEARCHING);
        assert!(c.challenger == Signer::address_of(signer), ERR_MUST_BE_CHALLENGER);
        let step_number = step_number(c);

        // assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        //assert!(c.challenger == Signer::address_of(signer), ERR_MUST_BE_CHALLENGER);

        let key_contains = Table::contains(&c.asserted_state, step_number);
        if (key_contains) {
            abort ERR_STATE_ALREADY_PROPOSED
        } else {
            Table::add(&mut c.asserted_state, step_number, state_hash);
        }
    }

    // the defender call this function to submit the state hash
    // for next step in the binary search
    public fun defend_state(sender: &signer, challenge_id: u64, state_hash: HashValue) acquires Challenges {
        let proposer_address = Signer::address_of(sender);
        assert!(is_searching(proposer_address, challenge_id), ERR_MUST_BE_SEARCHING);

        let challenges = borrow_global_mut<Challenges>(proposer_address);
        let c = Vector::borrow_mut(&mut challenges.value, challenge_id);
        let step_number = step_number(c);

        // assert!(c.challenger != @0x0, ERR_INVALID_CHALLENGE);
        // assert!(c.challenger == proposer_address, ERR_MUST_BE_CHALLENGER);


        // assert!(vector_to_u64(&hash_value::to_bytes(*asserted_state)) != 0, ERR_CHALLENGE_STATE_NOT_PROPOSED);

        // let defended_state = borrow_table_value(&c.defended_state, step_number);
        // assert!(vector_to_u64(&hash_value::to_bytes(*defended_state)) != 0, ERR_STATE_ALREADY_PROPOSED);

        let key_contains = Table::contains(&c.defended_state, step_number);
        if (key_contains) {
            abort ERR_STATE_ALREADY_PROPOSED
        } else {
            Table::add(&mut c.defended_state, step_number, state_hash);
        };

        // update binary search bounds
        let asserted_state = Table::borrow(&c.asserted_state, step_number);
        let defended_state = &state_hash;
        if (asserted_state == defended_state) {
            c.l = step_number;  // agree
        } else {
            c.r = step_number;  // disagree
        }
    }

    public fun deny_state_transition(sender: &signer, proposer: address, challenge_id: u64) acquires Challenges {
        let challenges = borrow_global<Challenges>(proposer);
        let c = Vector::borrow(&challenges.value, challenge_id);

        assert!(!is_searching_(c), ERR_BINARY_SEARCH_NOT_FINISHED);

        let asserted_state = Table::borrow(&c.asserted_state, c.l);
        let step_state = mips_emulator::run(Signer::address_of(sender), hash_value::to_bytes(*asserted_state));

        let right_asserted_state = Table::borrow(&c.asserted_state, c.r);
        assert!(step_state == hash_value::to_bytes(*right_asserted_state), ERR_WRONG_ASSERTED_STATE_FOR_CHALLENGER);

        // TODO: emit challenge wins event
    }

    public fun confirm_state_transition(sender: &signer, proposer_address: address, challenge_id: u64) acquires Challenges {
        let challenges = borrow_global<Challenges>(proposer_address);
        let c = Vector::borrow(&challenges.value, challenge_id);

        // if c.L + 1 == c.R, run the following code
        assert!(!is_searching_(c), ERR_BINARY_SEARCH_NOT_FINISHED);

        let defended_state = Table::borrow(&c.defended_state, c.l);
        let step_state = mips_emulator::run(Signer::address_of(sender), hash_value::to_bytes(*defended_state));

        let right_defended_state = Table::borrow(&c.defended_state, c.r);
        assert!(step_state == hash_value::to_bytes(*right_defended_state), ERR_WRONG_ASSERTED_STATE_FOR_DEFENDER);

        // TODO: emit defender wins event
    }
}


module omo::challenge_script {
    use omo::SimpleChallenge;
    use trie::hash_value;

    public(script) fun declare_state(signer: signer, final_state: vector<u8>) {
        SimpleChallenge::declare_state(&signer, hash_value::new(final_state));
    }
    public(script) fun create_challenge(signer: signer, proposer_address: address, final_system_state: vector<u8>, step_count: u64): u64 {
        SimpleChallenge::create_challenge(&signer, proposer_address, hash_value::new(final_system_state), step_count)
    }
    public(script) fun assert_state(signer: signer, proposer_address: address, challenge_id: u64, state_hash: vector<u8>) {
        SimpleChallenge::assert_state(&signer, proposer_address, challenge_id, hash_value::new(state_hash));
    }
    public(script) fun defend_state(signer: signer, challenge_id: u64, state_hash: vector<u8>) {
        SimpleChallenge::defend_state(&signer, challenge_id, hash_value::new(state_hash));
    }
    public(script) fun confirm_state_transition(sender: signer, proposer: address, challenge_id: u64) {
        SimpleChallenge::confirm_state_transition(&sender,proposer, challenge_id);
    }
    public(script) fun deny_state_transition(sender: signer, proposer_address: address, challenge_id: u64) {
        SimpleChallenge::confirm_state_transition(&sender,proposer_address, challenge_id);
    }
}

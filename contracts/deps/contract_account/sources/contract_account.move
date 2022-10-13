address ContractAddress{
    module ContractAccount {
        const CONTRACT_ACCOUNT:address = @ContractAddress;
        use StarcoinFramework::Account;

        struct GenesisSignerCapability has key {
            cap: Account::SignerCapability,
        }

        public fun initialize(signer:&signer, cap: Account::SignerCapability) {
            assert!(Account::signer_address(&cap) == CONTRACT_ACCOUNT, 1);
            move_to(signer, GenesisSignerCapability{cap});
        }

        public fun get_contract_signer(): signer acquires GenesisSignerCapability {
            let cap = borrow_global<GenesisSignerCapability>(CONTRACT_ACCOUNT);
            Account::create_signer_with_cap(&cap.cap)
        }

        public fun init_contract_account() {
            let genesis_account = Account::create_genesis_account(CONTRACT_ACCOUNT);
            let cap = Account::remove_signer_capability(&genesis_account);
            initialize(&genesis_account, cap);
        }
    }
}

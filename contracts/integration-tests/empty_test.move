//# init -n test

script {
    use StarcoinFramework::Signer;

    fun main(signer: signer) {
        StarcoinFramework::Debug::print(&Signer::address_of(&signer));
    }
}
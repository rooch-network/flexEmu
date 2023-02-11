## OMO Demo

Rust binary to demonstrate how omo work with onchain contracts to provide interacting fraud proof.

### How to run

#### Prerequisite

- Build a release binary of starcoin from dev branch, and put it into you PATH. I'll refer it as `starcoin`.
- Build a release binary of this repo. I'll refer it as `./target/release/omo-workflow`

#### Run

1. start starcoin dev server, and populate accounts and deploy onchain contracts.

```shell
cd ./omo-workflow
starcoin -n dev -d stc-dev
bash demo_init.sh
```

2. start defender and challenger.

cd root folder then run defender with:

```shell
./target/release/omo-workflow --nodeurl ws://127.0.0.1:9870 --config omo/config.toml.example --keyfile omo-workflow/defender-0x72d8f07846f8fc7efc742921310124b3.key --exec contracts/bins/arith-example --arg 1 --arg 10 proposer 4040
```

and challenger with:

```shell
./target/release/omo-workflow --nodeurl ws://127.0.0.1:9870 --config omo/config.toml.example --keyfile omo-workflow/challenger-0x613bcd14c23d993d3f751b218510a009.key --exec contracts/bins/arith-example --arg 1 --arg 10 challenger --proposer 0x72d8f07846f8fc7efc742921310124b3
```

this's it. defender and challenger will start an conversation to negotiate which step they have disagreement on.
and one can execute the final step to determine who is right.

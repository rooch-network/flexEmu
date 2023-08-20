#!/usr/bin/env bash
starcoin -c stc-dev/dev/starcoin.ipc account import -f ../contracts/dev.key
starcoin -c stc-dev/dev/starcoin.ipc account import -f ./challenger-0x613bcd14c23d993d3f751b218510a009.key
starcoin -c stc-dev/dev/starcoin.ipc account import -f ./defender-0x72d8f07846f8fc7efc742921310124b3.key

starcoin -c stc-dev/dev/starcoin.ipc dev get-coin -v 1000 0xfc2cd714a9d954dcec4ca5366d2461c4
starcoin -c stc-dev/dev/starcoin.ipc dev get-coin -v 1000 0x613bcd14c23d993d3f751b218510a009
starcoin -c stc-dev/dev/starcoin.ipc dev get-coin -v 1000 0x72d8f07846f8fc7efc742921310124b3

starcoin -c stc-dev/dev/starcoin.ipc account default 0xfc2cd714a9d954dcec4ca5366d2461c4
starcoin -c stc-dev/dev/starcoin.ipc account unlock 0xfc2cd714a9d954dcec4ca5366d2461c4

starcoin -c stc-dev/dev/starcoin.ipc dev deploy -b ../contracts/deps/signed_integer/release/signed_integer.v0.1.0.blob
starcoin -c stc-dev/dev/starcoin.ipc dev deploy -b ../contracts/deps/trie/release/trie.v0.1.0.blob
starcoin -c stc-dev/dev/starcoin.ipc dev deploy -b ../contracts/release/contracts.v0.0.0.blob

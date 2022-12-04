# Background of Layer2

### Bottleneck of Layer1 with Smart Contract

The most valuable resource on the chain is gas, because the gas of each block is capped, and the average block-out time
is fixed, so there is actually a limit to the number of computational steps that can be done per unit of time:

`TPS = (Max_Gas/Gas_Per_TXN)/Block-out_Time`

Obviously, we need to try to reduce gas per txn.

The direct way is to package multi txn into one, and that's the core idea of rollup.

### Optimistic Rollup

In last section, we've known that we need rollup, but who will take this responsibility, is it safe, how to challenge
the state?

First, we need to lock collateralized assets on chain for off-chain execution. Just like what we do in
Bitcoin Lightning Network.

Then, the verifier could be elected from community or DApp organization who deserves to be trusted. We are optimistic
that the chances of them doing evil are low because there is no obvious benefit.

The verifier provides abilities of persisting states on local for executing txn. What's more, we could set breakpoint
for finding wrong txn if there is one in challenge process.

In one word, Optimistic Rollup is a collateral-based off-chain solution to achieve scalability.

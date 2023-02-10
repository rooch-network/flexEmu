Background of Layer2
===

## Bottleneck of Layer1 with Smart Contract

The most valuable resource on the chain is gas, because the gas of each block is capped, and the average block-out time
is fixed, so there is actually a limit to the number of computational steps that can be done per unit of time:

`TPS = (Max_Gas_Per_Block/Gas_Per_TXN)/Block-out_Time`

Obviously, we need to reduce gas per txn.

The direct way is to package multi txn into one, and that's the core idea of rollup.


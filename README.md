omo
===

[中文](README.zh_CN.md)

A bytecode level program emulator with per-step state proof.

**Arch**: MIPS32 version 1 (SYSV) at present, will support more.

**Executable File Format**: ELF 32-bit MSB executable at present.

## Features

1. Crossing hardware platform (Coming soon)
2. Various smart chain virtual machine embed supports
3. Safe execution environment

## Background

### Limitation of Blockchain with Smart Contract

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

### Question: Is there any other way to improve TPS safely?

Yes! But OMO is just focused on Optimistic Rollup.

## Acknowledge

Inspired by [qiling](https://github.com/qilingframework/qiling).


# optimism-move

optimism-move 是一套基于 move 虚拟机的 layer2 的链上仲裁的原型实现，其基本设计主要沿用 [cannon](https://github.com/ethereum-optimism/cannon) 的思路。
目前还处在初期的计划实施阶段。
打通整个流程需要完成以下几个主要的模块。

### 裁剪&修改 starcoin 代码

这一步是为了形成一个通过 move-vm 执行 block 的最小代码集合。
其中主要涉及到，

- 剥离 move-vm 相关代码。
- 以及修改 vm 和 storage 交互逻辑。
- 将 stroage 接口 mock 掉，将数据读取和写入临时保存下来。

最终的目标是能够构建 min-move-vm，并且能够将其编译成 mips 指令集的二进制代码。
### 执行 mips 代码

将 min-move-vm 编译成 mips 指令集后，还需要一个 mips 代码的模拟执行环境，以执行该代码。
所以需要用 Rust 实现 min-move-vm mips代码的执行以及 vm 最终状态结果输出。
目前思路是利用 [unicorn](https://www.unicorn-engine.org/) 作为 mips 的基础执行环境。

### 链上合约实现

链上合约主要包括：

- mips 指令的执行。主要难点在于实现 memory 和 state 的读取，写入，这涉及到在链上实现一个 merkle tree，以验证读取的正确性。
- 交互式仲裁的实现。这一部分可以参考 cannon 的 逻辑，将其用 move 实现。

### 模块串联

三个模块完成之后，需要串联起来，跑通整个流程。

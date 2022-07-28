# OMO 介绍 - Part 1

## 背景

随着链上交易量增多，layer2 成为解决扩容问题的一个重要研究领域，其中 optimistic rollup 这一方向已经有了落地的解决方案，比如 arbitrum(后称 arb),optimism(后称 op)。

arb 和 op（最新实现的 cannon） 都采用了交互式的链上仲裁技术。交互式的意思是，挑战者在一层不断的挑战出块者在执行区块时产生的中间状态，直到双方在某一个中间指令的执行结果上出现分歧，抑或是最终没有分歧。这种仲裁手段要求 layer2 的虚拟机能够保存执行区块过程中产生的中间状态的证明，抑或是能够在某次指令执行时进行中断，并保存当前状态的证明。

因为不同的optimistic rollup 解决方案可能使用了不同的虚拟机，比如， arbitrum 自己研发了 AVM。为了生成中间状态证明，不同的虚拟机可能需要定制自己的状态证明技术，且通用性比较差。虽然 optimistic rollup 的技术原理基本上都是一样的，但现如今，公链多点开花，应用也百花齐放，智能合约语言也呈快速演进趋势，为某个公链 X 实现基于某种智能合约语言 Y 的 optimistic rollup，仍然需要在工程中付出很多重复性的工作。

基于此，我们试图实现一个通用的可验证的合约执行环境。该环境基于 Linux 系统，可模拟不同 CPU 指令架构的代码执行，并生成对应的中间状态（寄存器+内存数据）以及状态证明。整体思路受 cannon 的启发，并借鉴了 qiling 框架的部分实现。

和 qiling 不同的是，区块链中的合约执行环境相对简单，没有 IO 等复杂的外部交互，也无需考虑系统安全性。另外，qiling 缺少中间状态生成以及状态证明，这是做交互式仲裁所必须的功能之一。

我们把这个合约执行环境称作 OMO。

## 主要构件

OMO 主要由以下几个组件构成：

- 执行环境。对 unicorn 封装后的数据结构，提供了代码执行中对内存和寄存器的操作接口。
- 加载器。用来加载代码数据到执行环境中。目前主要支持 ELF 的加载。
- 寄存器和数据栈。主要便于接口抽象。
- 系统接口。主要实现了 linux 下的 syscall。

## 执行环境

unicorn 是底层用来真正执行代码的工具库。我们利用其中的数据结构加上抽闲所需要的额外结构 `Machine<A>` 来代表整个执行环境，也就是运行中所需要的数据。

其中 `Machine<A>` 包含了两部分信息：

- `memories: MemoryManager` 用来管理运行时内存分配的信息。
- `arch: A` 代表了底层的CPU 架构信息，这里通过范型表示。目前实现了 MIPS 架构的 arch，其他 CPU 指令集比如 riscv 也可以比较容易的扩展进去。

```rust
pub type Engine<'a, A> = Unicorn<'a, Machine<A>>;

#[derive(Debug)]
pub struct Machine<A> {
    pub(crate) memories: MemoryManager,
    arch: A,
}
```

## Code Loader

代码加载器目前支持了 ELF loader。其完整模拟了 linux 下 elf 代码的加载过程：[How programs get run: ELF binaries](https://lwn.net/Articles/631631/) 。

其主要实现：

- 解析出代码和数据段，加载进内存。
- 初始化代码的入参，以及环境变量。
- 初始化栈指针以及程序计数器（即代码入口）。

完整加载后，整个程序即处于可运行状态。

## Register and Stack

底层 Unicorn 库只提供了通用的读写内存和寄存器的的接口。

在此之上， OMO 提供了关于栈的抽象，以及更高级的寄存器和内存操作。

具体接口如下（随着项目演化，可能会略有变动）。

```rust
pub trait Registers {
    fn read(&self, reg: impl Into<i32>) -> Result<u64, uc_error>;
    fn write(&mut self, reg: impl Into<i32>, value: u64) -> Result<(), uc_error>;
    fn pc(&self) -> Result<u64, uc_error>;
    fn set_pc(&mut self, value: u64) -> Result<(), uc_error>;
}
pub trait StackRegister {
    fn sp(&self) -> Result<u64, uc_error>;
    fn set_sp(&mut self, value: u64) -> Result<(), uc_error>;

    /// increment stack pointer by `delta`.
    /// Return new stack pointer
    fn incr_sp(&mut self, delta: i64) -> Result<u64, uc_error> {
        let cur = self.sp()?;
        let new_sp = cur.checked_add_signed(delta).ok_or(uc_error::EXCEPTION)?;
        self.set_sp(new_sp)?;
        Ok(new_sp)
    }
}
```

```rust
pub trait Memory {
    fn pagesize(&self) -> u64 {
        PAGE_SIZE as u64
    }
    fn mem_map(&mut self, region: MemRegion, info: Option<String>) -> Result<(), uc_error>;
    fn mprotect(&mut self, addr: u64, size: usize, perm: Permission) -> Result<(), uc_error>;
    fn mem_unmap(&mut self, addr: u64, size: usize) -> Result<(), uc_error>;
    fn is_mapped(&self, addr: u64, size: usize) -> Result<bool, uc_error>;
    fn read(&self, addr: u64, size: usize) -> Result<Vec<u8>, uc_error>;
    fn read_ptr(&self, address: u64, pointersize: Option<PointerSizeT>) -> Result<u64, uc_error>;

    fn write(&mut self, address: u64, bytes: impl AsRef<[u8]>) -> Result<(), uc_error>;
    fn write_ptr(
        &mut self,
        address: u64,
        value: u64,
        pointersize: Option<PointerSizeT>,
    ) -> Result<(), uc_error>;
}
```

这些接口是直接实现在 ExecEnv 数据结构上。

pc，sp 等寄存器的地址和 CPU 指令架构相关，所以 ExecEnv 中有关于 arch 抽象的信息。

## System Call

要运行的代码依赖 linux 下的各种系统调用。OMO 需要从底层模拟这些系统接口，并保证接口调用是幂等的。

linux 的系统调用比较多，目前在一一梳理和实现 https://github.com/starcoinorg/omo/issues/6。

不过运行  hello world 所需要的系统调用已经完成。有兴趣的朋友可以参考 [https://github.com/starcoinorg/omo/tree/main/rust-mips-example](https://github.com/starcoinorg/omo/tree/main/rust-mips-example) 这个例子来运行。

本文主要介绍了 omo 项目的背景和实现中的几个核心组件，希望能够帮助读者了解 omo。

下一篇，我们会介绍如何生成中间状态及其状态证明。
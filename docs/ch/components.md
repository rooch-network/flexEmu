Components
===

flexEmu 主要有以下四个关键组件：

* Loader: 加载可执行文件。
* CPU Emulator: 在宿主机上模拟执行指令。
* Registers and Stack: 寄存器与栈模拟。
* OS Interface: 在宿主机上模拟执行系统调用。

![architecture](../imgs/arch.png)

## Loader

代码加载器目前支持了 ELF loader。其完整模拟了 linux 下 elf 代码的加载过程：[How programs get run: ELF binaries](https://lwn.net/Articles/631631/) 。

其主要实现：

- 解析出代码和数据段，加载进内存。
- 初始化代码的入参，以及环境变量。
- 初始化栈指针以及程序计数器（即代码入口）。

完整加载后，整个程序即处于可运行状态。

## CPU Emulator

基于 [Unicorn](https://github.com/unicorn-engine/unicorn) 实现了支持多 CPU 指令集与管理内存的能力。

## Registers and Stack

在 CPU Emulator 之上， flexEmu 提供了关于栈的抽象，以及更高级的寄存器和内存操作。这样我们可以很方便的对运行状态进行快照。

## OS Interface

flexEmu 模拟了大量系统调用，并保证调用是幂等的。

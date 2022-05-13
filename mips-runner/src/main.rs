use unicorn_engine::Unicorn;
use unicorn_engine::unicorn_const::{Arch, Mode};

fn main() {
    let engine = Unicorn::new(Arch::MIPS,Mode::MIPS32).unwrap();
}

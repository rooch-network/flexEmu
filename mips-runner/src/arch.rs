use goblin::container::Endian;

pub struct Arch {
    pub type_: unicorn_engine::unicorn_const::Arch,
    pub bits: u8,
    pub endian: Endian,
}

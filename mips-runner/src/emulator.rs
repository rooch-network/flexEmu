use crate::arch::Arch;

pub struct Emulator<'a, Loader, Os> {
    arch: Arch<'a>,
    loader: Loader,
    os: Os,
}

impl<'a, Loader, Os> Emulator<'a, Loader, Os>{}





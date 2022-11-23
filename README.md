# libctr-rs

This is a rust version of libctru, with some C bindings to libctru.

The goals of this project are to:

- Get a good rust IPC system in place for rust sysmodules (which is part of the reason `ctru_sys` isn't used everywhere)
- Provide a way to compile code on any machine by stubbing `unimplemented!` when not compiling for Horizon
- Fix a few issues with pure libctru C bindings (some structs, such as `Mii`, aren't currently compatible with rust, and split arm/thumb builds aren't possible, which causes issues with the inlined IPC functions)

## Assumptions and contribution requirements

This crate has a few assumptions about its host:

- Any Horizon specific logic will be `unimplemented!` outside Horizon to allow unit testing on any machine
- The host machine is at least a 32 bit host

Requirements:

- All code must support 64 bit machines, as those are common to run tests on
- All code should be agnostic to the host endianness
- All structs should derive `Clone`, `Copy`, `Debug`, `PartialEq`, and `Default` when possible

## Credits

Thanks to these projects, teams, and individuals for being great resources:

- [libctru](https://github.com/devkitPro/libctru/) for being a great reference, providing an easy way to make open source hombrew, and where much of the code came from
- [Luma3ds](https://github.com/LumaTeam/Luma3DS) for being a great reference and providing custom svcs
- [3dbrew](https://www.3dbrew.org/) and [citra](https://github.com/citra-emu/citra) for documentation about how different parts of the 3ds work
- [The rust3ds team](https://github.com/rust3ds) for the 3ds.json, initial ctru_sys, and code references to help get rust working on the 3ds
- [devkitPro](https://github.com/devkitPro/) for their toolchain
- All 3ds researchers

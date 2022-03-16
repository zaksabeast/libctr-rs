# libctr-rs

This is a rust version of libctru, with some C bindings to libctru.

The goals of this project are to:

- Get a good rust IPC system in place for rust projects
- Provide a way to mock libctru behavior so tests can be written for homebrew and run on any machine
- Fix a few issues with pure libctru C bindings (some structs, such as `Mii`, aren't currently compatible with rust, and split arm/thumb builds aren't possible, which causes issues with the inlined IPC functions)

Whether this borrows more from libctru in the future or removes libctru/ctry_sys as a dependency in the future is unknown at this time.

## Mocks and testing

Unit tests are important to ensure changes don't accidentally break functionality and to serve as documentation. Most 3ds homebrew does not include unit tests due to 3ds specific mechanics.

One of this crate's goals is to provide mock functions with stubbed responses when the target OS is not Horizon. Because the libctr-rs mocks are target agnostic and ctru_sys is not included when the target OS is not Horizon, any homebrew using this library should be able to write test that will run on any target.

When writing tests that require a libctr-rs function to have a specific return value (e.g. an error), mocktopus can be used to mock whatever response is needed.

## Assumptions and contribution requirements

This crate has a few assumptions about its host:

- Any code not compiled for Horizon must be mocks for tests
- The host machine is at least a 32 bit host

Requirements:

- All code must support 64 bit machines, as those are common to run tests on
- All code should be agnostic to the host endianness
- All structs should derive `Clone`, `Copy`, `Debug`, `PartialEq`, and `Default` when possible
- All structs should be `repr(C)` or `repr(primitive)`
- All impls and functions should be mockable with mocktopus

## Credits

Thanks to these projects, teams, and individuals for being great resources:

- [libctru](https://github.com/devkitPro/libctru/) for being a great reference and providing an easy way to make open source hombrew
- [3dbrew](https://www.3dbrew.org/) and [citra](https://github.com/citra-emu/citra) for documentation about how different parts of the 3ds work
- [The rust3ds team](https://github.com/rust3ds) for the 3ds.json, initial ctru_sys, and code references to help get rust working on the 3ds
- [devkitPro](https://github.com/devkitPro/) for their toolchain
- All 3ds researchers

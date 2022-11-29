CARGO_BUILD_FLAGS = -Z build-std=core,alloc --target armv6k-nintendo-3ds

SOURCES = $(wildcard src/*.rs) $(wildcard src/**/*.rs) $(wildcard src/**/**/*.rs)

.PHONY: lint docs test clean

release : $(SOURCES)
	@cargo +nightly build --release $(CARGO_BUILD_FLAGS)
	
debug : $(SOURCES)
	@cargo +nightly build $(CARGO_BUILD_FLAGS)

lint:
	@cargo +nightly clippy -Z unstable-options $(CARGO_BUILD_FLAGS)

docs:
	@cargo +nightly doc --open

test:
	@cargo +nightly test

clean:
	@cargo clean

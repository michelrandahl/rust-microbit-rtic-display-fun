LAST_DIR_NAME := $(notdir $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST))))))

USE_RTT_RUSTFLAGS = "-C linker=flip-link -C link-arg=-Tlink.x"
USE_DEFMT_RUSTFLAGS = "-C linker=flip-link -C link-arg=-Tlink.x -C link-arg=-Tdefmt.x"

clean:
	cargo clean

docs:
	cargo doc --open --target thumbv7em-none-eabihf --features use_defmt

build-rtt:
	RUSTFLAGS=$(USE_RTT_RUSTFLAGS) cargo build --target thumbv7em-none-eabihf --features use_rtt

build-defmt:
	RUSTFLAGS=$(USE_DEFMT_RUSTFLAGS) cargo build --target thumbv7em-none-eabihf --features use_defmt
build-release:
	RUSTFLAGS=$(USE_DEFMT_RUSTFLAGS) cargo build --target thumbv7em-none-eabihf --release --features use_defmt

test:
	cargo test --target x86_64-unknown-linux-gnu

check-defmt:
	RUSTFLAGS=$(USE_DEFMT_RUSTFLAGS) cargo check --target thumbv7em-none-eabihf --features use_defmt

check-rtt:
	RUSTFLAGS=$(USE_RTT_RUSTFLAGS) cargo check --target thumbv7em-none-eabihf --features use_rtt

embed: build-rtt
	RUSTFLAGS=$(USE_RTT_RUSTFLAGS) cargo embed --target thumbv7em-none-eabihf --features use_rtt

probe-run: build-defmt
	probe-run --chip nrf52833_xxAA target/thumbv7em-none-eabihf/debug/$(LAST_DIR_NAME)

probe-run-release: build-release
	probe-run --chip nrf52833_xxAA target/thumbv7em-none-eabihf/release/$(LAST_DIR_NAME)

serial:
	picocom /dev/ttyACM0 -b 115200 --omap crcrlf

size:
	RUSTFLAGS=$(USE_DEFMT_RUSTFLAGS) cargo size --target thumbv7em-none-eabihf --features use_defmt -- -A
size-release:
	RUSTFLAGS=$(USE_DEFMT_RUSTFLAGS) cargo size --target thumbv7em-none-eabihf --features use_defmt --release -- -A

readobj:
	cargo readobj --target thumbv7em-none-eabihf --bin $(LAST_DIR_NAME) -- --file-headers
readobj-release:
	cargo readobj --release --target thumbv7em-none-eabihf --bin $(LAST_DIR_NAME) -- --file-headers

gdb:
	gdb ./target/thumbv7em-none-eabihf/debug/$(LAST_DIR_NAME) -x ./.gdbinit

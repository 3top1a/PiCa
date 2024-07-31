CARGO := cargo
RUSTFLAGS := -C target-cpu=native

.PHONY: all build run clean test bench

all: build

build:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) build --release

run:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) run --release

clean:
	$(CARGO) clean

test:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) test

bench:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) run --release -- --bench

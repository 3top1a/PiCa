CARGO := cargo
RUSTFLAGS := -C target-cpu=native

.PHONY: all build run clean test

all: build

build:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) build --release

run: build
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) run --release

clean:
	$(CARGO) clean

test:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) test

CARGO := cargo
RUSTFLAGS := -C target-cpu=native  -C symbol-mangling-version=v0 -C target-feature=-bmi2

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

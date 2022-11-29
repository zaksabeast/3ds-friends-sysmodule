CRATE_NAME	= friends_sysmodule

CARGO_BUILD_FLAGS = -Z build-std=core,alloc

RUST_OUT_DIR	=	target/armv6k-nintendo-3ds
RUST_RELEASE_DIR	=	$(RUST_OUT_DIR)/release
RUST_DEBUG_DIR	=	$(RUST_OUT_DIR)/debug

OUT_DIR	= out
RELEASE_DIR = $(OUT_DIR)/release/0004013000003202
DEBUG_DIR = $(OUT_DIR)/debug/0004013000003202

RELEASE_ELF	= $(RUST_RELEASE_DIR)/$(CRATE_NAME).elf
DEBUG_ELF	= $(RUST_DEBUG_DIR)/$(CRATE_NAME).elf

RELEASE_CXI	= $(RUST_RELEASE_DIR)/$(CRATE_NAME).cxi
DEBUG_CXI	= $(RUST_DEBUG_DIR)/$(CRATE_NAME).cxi

SOURCES = $(wildcard src/*.rs) $(wildcard src/**/*.rs) $(wildcard src/**/**/*.rs)

.PHONY: all clean test docs docs lint

all: release debug test

docs:
	@cargo doc --open $(CARGO_BUILD_FLAGS)

# Nightly and unstable options prevent clippy from linting dependencies - https://github.com/rust-lang/rust-clippy/issues/1066
lint:
	@cargo +nightly clippy -Zunstable-options $(CARGO_BUILD_FLAGS) --target armv6k-nintendo-3ds

test:
	@cargo test

release: $(RELEASE_DIR)/exheader.bin

debug: $(DEBUG_DIR)/exheader.bin

$(RELEASE_DIR)/exheader.bin : $(RELEASE_CXI)
	@ctrtool --exefsdir=$(RELEASE_DIR) --exheader=$@ $< > /dev/null
	@echo Built code and exheader

$(DEBUG_DIR)/exheader.bin : $(DEBUG_CXI)
	@ctrtool --exefsdir=$(DEBUG_DIR) --exheader=$@ $< > /dev/null
	@echo Built code and exheader

$(RELEASE_CXI) : $(RELEASE_ELF)
	@mkdir -p $(RELEASE_DIR)
	@makerom -f ncch -rsf friends-release.rsf -o $@ -elf $<

$(DEBUG_CXI) : $(DEBUG_ELF)
	@mkdir -p $(DEBUG_DIR)
	@makerom -f ncch -rsf friends-debug.rsf -o $@ -elf $<

$(RELEASE_ELF) : $(SOURCES)
	@cargo build --release $(CARGO_BUILD_FLAGS) --target armv6k-nintendo-3ds

$(DEBUG_ELF) : $(SOURCES)
	@cargo build $(CARGO_BUILD_FLAGS) --target armv6k-nintendo-3ds

clean:
	@rm -rf $(OUT_DIR)
	@cargo clean

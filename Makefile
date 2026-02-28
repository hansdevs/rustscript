.PHONY: build release clean install

# Default: build for the current platform (release mode)
build:
	cd rustscript && cargo build --release

# Install to ~/.cargo/bin (or override with INSTALL_DIR)
INSTALL_DIR ?= $(HOME)/.cargo/bin
install: build
	@mkdir -p $(INSTALL_DIR)
	cp rustscript/target/release/rustscript $(INSTALL_DIR)/rustscript
	@echo "✓ Installed to $(INSTALL_DIR)/rustscript"

# Build for all release platforms (requires cross-compilation targets installed)
TARGETS = x86_64-apple-darwin aarch64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-msvc

release:
	@mkdir -p dist
	@for target in $(TARGETS); do \
		echo "Building $$target..."; \
		cd rustscript && cargo build --release --target $$target && cd ..; \
		case $$target in \
			*windows*) cp rustscript/target/$$target/release/rustscript.exe dist/rustscript-$$(echo $$target | sed 's/-unknown-linux-gnu//' | sed 's/-apple-darwin//' | sed 's/-pc-windows-msvc//').exe ;; \
			*darwin*)  bn=$$(echo $$target | sed 's/-unknown//;s/-pc//'); cp rustscript/target/$$target/release/rustscript dist/rustscript-darwin-$$(echo $$target | cut -d- -f1) ;; \
			*linux*)   cp rustscript/target/$$target/release/rustscript dist/rustscript-linux-$$(echo $$target | cut -d- -f1) ;; \
		esac; \
	done
	@echo "✓ Binaries in dist/"

clean:
	cd rustscript && cargo clean
	rm -rf dist

.PHONY: compile
compile:
	docker run --rm -it \
		-v $(PWD):/mlc \
		-w /mlc \
		joseluisq/rust-linux-darwin-builder:1.45.1 \
		make cross-compile

.PHONY: cross-compile
cross-compile:
	@echo
	@echo "1. Cross compiling"
	@rustc -vV
	@echo
	@echo "2. Compiling application (linux-musl x86_64)..."
	@cargo build --release --target x86_64-unknown-linux-musl
	@du -sh target/x86_64-unknown-linux-musl/release/mlc
	@echo
	@echo "3. Compiling application (apple-darwin x86_64)..."
	@cargo build --release --target x86_64-apple-darwin
	@du -sh target/x86_64-apple-darwin/release/mlc

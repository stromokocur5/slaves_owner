default: build test

build:
	cargo build

test:
	cargo test

watch: 
	cargo watch -s just


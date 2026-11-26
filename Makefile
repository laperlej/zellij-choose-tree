dev:
	@zellij action new-tab --layout ./plugin-dev-workspace.kdl

clean:
	@pkill watchexec

deploy:
	cargo build --target wasm32-wasip1 --release
	@cp $(shell pwd)/target/wasm32-wasip1/release/zellij-choose-tree.wasm ~/.config/zellij/plugins

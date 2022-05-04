cargo build --release
cp target/release/piranha-tree-sitter  demo/java/piranha-tree-sitter
./demo/java/piranha-tree-sitter -c demo/java/ -f demo/java/configurations -p demo/java/demo_piranha_arguments.toml
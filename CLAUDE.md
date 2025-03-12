# CLAUDE.md - Sugi-img Project Guidelines

## Build/Lint/Test Commands
- Build: `cargo build --release`
- Run Rust: `cargo run --release -- --input-dir="data/受領画像" --output-dir="output/compressed" --quality=90`
- Debug Run: `cargo run -- --input-dir="data/受領画像" --log-level=debug`
- Format: `cargo fmt`
- Lint: `cargo clippy`
- Check: `cargo check`
- Test: `cargo test`
- Single Test: `cargo test <test_name> -- --nocapture`

## Code Style Guidelines
- **Imports**: Order by 1) external crates, 2) standard library, 3) internal modules
- **Naming**: snake_case for variables/functions, CamelCase for types/traits, UPPERCASE for constants
- **Documentation**: Use doc comments (//!) for modules and (///) for functions
- **Error Handling**: Use anyhow::Result with context() for meaningful error messages
- **Types**: Use PathBuf for file paths, explicit types for clarity
- **Logging**: log::{info, warn, debug} with appropriate detail levels
- **Module Structure**: Separate concerns (cli, logger, compressor)
- **Parallelism**: Use rayon with appropriate thread count (num_cpus::get() / 2)
- **Resource Management**: Ensure proper cleanup with RAII pattern
- **File Handling**: Check existence before operations, maintain directory structure

This project compresses JPEG images with configurable quality settings while preserving directory structure.
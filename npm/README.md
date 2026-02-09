# claude-explorer

A terminal-based file explorer designed to work alongside Claude Code CLI. View your project structure in a split-pane TUI while using Claude Code.

## Installation

```bash
npm install -g claude-explorer
# or
bun install -g claude-explorer
```

## Usage

```bash
# Start in current directory
claude-explorer

# Start in specific directory
claude-explorer --path /path/to/project

# Adjust tree width (10-50%)
claude-explorer --tree-width 25

# Show hidden files
claude-explorer --show-hidden
```

## How it works

This package downloads the pre-built native binary for your platform from [GitHub Releases](https://github.com/jsleemaster/claude-explorer/releases) during installation. No Rust toolchain required.

### Supported platforms

- macOS (Apple Silicon / Intel)
- Linux (x86_64 / ARM64)
- Windows (x86_64)

## Alternative installation methods

```bash
# From crates.io
cargo install claude-explorer

# Homebrew (macOS/Linux)
brew install jsleemaster/tap/claude-explorer
```

## License

MIT - see [LICENSE](https://github.com/jsleemaster/claude-explorer/blob/main/LICENSE)

# fledge-plugin-quiz

Developer trivia quiz for fledge -- 63 questions across HTTP, Git, algorithms, shell, Rust, and more.

A terminal-based quiz plugin that tests your developer knowledge with color-coded difficulty levels, randomized question order, and a progress bar score summary.

## Install

```bash
fledge plugins install corvid-agent/fledge-plugin-quiz
```

Requires **fledge >= 0.15.3** (which sets `FLEDGE_PLUGIN_DIR` automatically).

## Usage

```
fledge quiz <command> [options]
```

### Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `play [topic]` | `p` | Answer questions from a topic (random if omitted) |
| `topics` | `t` | List available topics with question counts |
| `random` | `r` | Single random question from any topic |
| `scores` | `s` | Score tracking (coming in v0.3.0) |

### Options

| Flag | Description |
|------|-------------|
| `--count <n>` | Number of questions per session (default: 5) |
| `--hard` | Only hard-difficulty questions |

### Examples

```bash
# Quick 5-question quiz on a random topic
fledge quiz play

# 10 HTTP questions
fledge quiz play http --count 10

# Hard-mode Rust quiz
fledge quiz play rust --hard

# One random question (good for a break)
fledge quiz random

# See all topics and difficulty breakdowns
fledge quiz topics
```

## Topics

| Topic | Questions | Covers |
|-------|-----------|--------|
| algorithms | 12 | Big-O, sorting, data structures, graph algorithms |
| general | 12 | Developer fundamentals, networking, encoding |
| git | 12 | Porcelain commands, internals, refs, rebase |
| http | 15 | Status codes, methods, headers, caching |
| rust | 12 | Ownership, lifetimes, traits, async, macros |
| shell | 12 | Bash builtins, pipes, signals, file descriptors |

Each question has a difficulty rating: **easy** (green), **medium** (yellow), or **hard** (red).

## Development

Built in Rust with [crossterm](https://crates.io/crates/crossterm) for terminal colors and [rand](https://crates.io/crates/rand) for shuffling.

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

### Adding questions

Add or edit JSON files in `questions/`. Each file is a JSON array of objects:

```json
[
  {
    "question": "What does HTTP 418 mean?",
    "answer": "I'm a teapot",
    "difficulty": "easy"
  }
]
```

Valid difficulty values: `easy`, `medium`, `hard`. If omitted, defaults to `medium`.

## License

MIT

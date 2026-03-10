# atcoder-tui-viewer

Terminal viewer for AtCoder task pages with Markdown-like rendering.

## Install

```bash
cargo install --git https://github.com/hxrxchang/atcoder-tui-viewer --bin atv
```

Update to latest:

```bash
cargo install --git https://github.com/hxrxchang/atcoder-tui-viewer --bin atv --force
```

## Usage

```bash
cargo run -- view "https://atcoder.jp/contests/abc357/tasks/abc357_d"
```

English:

```bash
cargo run -- view "https://atcoder.jp/contests/abc357/tasks/abc357_d" --lang en
```

## Keys

- `q`: quit
- `j` / `Down`: scroll down
- `k` / `Up`: scroll up
- `PageDown` / `PageUp`: fast scroll

# atcoder-tui-viewer

Terminal viewer for AtCoder problem pages with Markdown-like rendering.

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
atv view "https://atcoder.jp/contests/abc357/tasks/abc357_d"
```

English:

```bash
atv view "https://atcoder.jp/contests/abc357/tasks/abc357_d" --lang en
```

## Keys

- `q`: quit
- `j` / `Down`: scroll down
- `k` / `Up`: scroll up
- `PageDown` / `PageUp`: fast scroll

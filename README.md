# wsl-open

A minimal WSL utility to open files, URLs, and domains using Windows default applications via `cmd.exe start`.

## Features

- Open HTTP/HTTPS URLs
- Open domain names (auto-prepend `https://`)
- Open local files and directories from WSL
- Convert WSL paths to Windows paths via `wslpath`
- Supports `~/` expansion

## Behavior

Input is resolved in the following order:

1. URL (`http://` or `https://`)
2. Domain (`example.com`, `localhost`, `localhost:3000`)
3. Local path (if exists)
4. Error (unsupported input)

## Requirements

- WSL (tested on WSL2)
- Windows `cmd.exe`
- `wslpath` available in PATH

## Build

```bash
cargo build --release
```

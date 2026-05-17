# wsl-open

A simple utility to open files, URLs, and domains from WSL using the Windows default application.

## Features

- Open web URLs
- Open domain names (e.g. example.com)
- Open local files and directories
- Supports paths inside WSL
- Supports `~/` home directory expansion

## Usage

```bash
wsl-open https://github.com
wsl-open example.com
wsl-open ./src/main.rs
wsl-open ~/Pictures/image.png
```

## Behavior

Input will automatically be handled based on its type:

- URL → opened in default browser
- Domain → treated as HTTPS URL
- File or directory → opened in Windows default application

## Requirements

- WSL environment
- Windows system integration

## License

MIT

# QuoteKeeper

QuoteKeeper is a compact, Rust-based tool designed to quickly capture and organize quotes. Ideal for academic, literary, or personal use, it stores quotes efficiently in a JSON format.

> [!NOTE]
> This project has not been thoroughly tested. Bugs might arise.

## Why use QuoteKeeper?

Imagine you're in a history class with your eccentric teacher, and they say something funny that you'd like to save for later. Instead of wasting 45 seconds opening a new file and filling out the details, QuoteKeeper allows you to jot it down instantly and keep it neatly organized.

## Compatibility

- Linux: Supported
- macOS: Compatibility unknown
- Windows: Not supported


## How to install?


> [!NOTE]
> Make sure to add `export PATH="$HOME/.cargo/bin:$PATH"` to your shell configuration file (e.g., `.bashrc`, `.zshrc`) to ensure that binaries installed with `cargo` are available system-wide.

Using cargo:
```bash
cargo install --git https://github.com/Urpagin/quotekeeper.git --branch master
```

## How to completely uninstall?

Removing files + uninstalling with cargo:

```bash
rm -rf ~/.quotekeeper/ && cargo uninstall quotekeeper
```
## Where are the quotes stored?

All data related to QuoteKeeper is located in the home directory:

- **Quotes:** The quotes you save are stored in `~/.quotekeeper/quotes.json`.
- **Configuration:** Settings can be adjusted in `~/.quotekeeper/config.conf`.

## Features

- Select your preferred editor or even use stdin for recording quotes. This setting can be adjusted in the config file located at `~/.quotekeeper/config.conf`.

- Automatically backs up your quotes file if the JSON somehow gets corrupted.


## Known Issues

When using a custom editor, if the startup time is too long, the program may not function as expected. This issue is particularly noticeable with VSCodium.

## Todo

- [ ] Implement cli commands to change the settings
- [ ] Most recent quote authors system. (less typing)
- [ ] Add other export format (e.g. .txt)
- [ ] Delete & modify records

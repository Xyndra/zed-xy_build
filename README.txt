xy-build Language Extension for Zed
====================================

This extension provides language support for the xy-build build system,
including LSP integration for code intelligence features.


How the LSP Binary is Obtained
------------------------------

By default, the extension downloads the LSP binary from GitHub releases:
https://github.com/xyndra/xy_build/releases

The extension automatically detects your platform and downloads the appropriate
binary for:
  - Windows x64 (x86_64-pc-windows-msvc)
  - Linux x64 (x86_64-unknown-linux-gnu)
  - Linux ARM64 (aarch64-unknown-linux-gnu)
  - macOS x64 (x86_64-apple-darwin)
  - macOS ARM64 (aarch64-apple-darwin)


Local Development
-----------------

When building the LSP server locally for development, you can place the binary
in your project directory:

  cargo build --release -p xy-build-lsp

After the build completes, copy the binary to this extensions's root. Linux example:

  cp ../target/release/xy-build-lsp ~/.local/share/zed/extensions/work/xy_build/xy-build-lsp

The extension checks for a ./xy-build-lsp binary first, and if found, uses it
instead of the downloaded version. This allows you to test changes without
affecting the released version.

To revert to the released version, delete the local xy-build-lsp binary file
and restart Zed. The extension will then download the latest release from GitHub.


Updating the LSP Version
------------------------

The extension fetches the latest release from GitHub automatically. To use a
specific version, place the binary in your project directory as described above.

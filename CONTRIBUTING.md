# Contributing to speech-flow

speech-flow is Hagan McPhail's speech-to-text app, forked from [Handy](https://github.com/cjpais/Handy). Changes in this repository are for speech-flow. Handy's feature freeze and pull request rules do not apply here.

## Development setup

You need [Rust](https://rustup.rs/) and [Bun](https://bun.sh/). Platform build tools are in [BUILD.md](BUILD.md).

```bash
bun install

mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx

bun run tauri dev
```

On macOS, if CMake fails during that command:

```bash
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev
```

The layout of the Rust and React code is described in [AGENTS.md](AGENTS.md).

## Bugs and ideas

Open them on this repository. Use the bug report template when it fits. A feature request can be an issue or a pull request here. You do not need a discussion on the upstream Handy repo first.

## Pull requests

Use the template in `.github/PULL_REQUEST_TEMPLATE.md`:

- **Summary** — what the change is for
- **Changes** — what you actually changed
- **Testing** — what you ran, and what still needs a check on macOS when the change touches the hotkey, permissions, paste, or the clipboard

Commit messages use conventional prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`) and should say why the change exists.

## Code style

**Rust:** `cargo fmt` before committing. Prefer `cargo clippy` clean. Handle errors explicitly, and add doc comments on public APIs.

**TypeScript:** Strict types, functional components, Tailwind for styling. User-facing strings go through i18next. See [CONTRIBUTING_TRANSLATIONS.md](CONTRIBUTING_TRANSLATIONS.md) for new translations.

## License

By contributing, you agree that your contributions are licensed under the MIT License. See [LICENSE](LICENSE). That file also retains the original Handy copyright.

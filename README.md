# Tauri + SvelteKit + TypeScript

This template should help get you started developing with Tauri, SvelteKit and TypeScript in Vite.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Contributing

Before opening a merge request, please ensure your code passes the following checks.

### Frontend

Run these commands to verify the Svelte/TypeScript code:

```bash
npx prettier --check .

deno task check
```

### Backend

Run these commands to verify the Rust code:

```bash
cd src-tauri

cargo clippy && cargo test && cargo fmt -- --check
```

### Verification

The verification workflow will automatically run these checks on any pull request. You can run the full build locally to ensure everything works:

```bash
deno task build
cargo build
```

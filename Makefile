.PHONY: check format check-frontend check-backend format-frontend format-backend test build dev preview build-frontend build-backend tauri check-watch

# Run all checks (frontend and backend)
check: check-frontend check-backend

# Format all code (frontend and backend)
format: format-frontend format-backend

# Frontend verification
check-frontend:
	npx prettier --check .
	npx svelte-kit sync && npx svelte-check --tsconfig ./tsconfig.json

check-watch:
	npx svelte-kit sync && npx svelte-check --tsconfig ./tsconfig.json --watch

format-frontend:
	npx prettier --write .

# Backend verification
# Using --manifest-path to run cargo commands from root without cd
check-backend:
	cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
	cargo clippy --manifest-path src-tauri/Cargo.toml
	cargo test --manifest-path src-tauri/Cargo.toml

format-backend:
	cargo fmt --manifest-path src-tauri/Cargo.toml

# Run backend tests specifically
test:
	cargo test --manifest-path src-tauri/Cargo.toml

# Build application
build: build-frontend build-backend

build-frontend:
	npm run build

build-backend:
	cargo build --manifest-path src-tauri/Cargo.toml

# Dev server
dev:
	npm run tauri dev

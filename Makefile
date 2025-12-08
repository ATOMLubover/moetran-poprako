.PHONY: fmt
fmt:
	@echo === formatting frontend... ===

	pnpm format

	@echo --- frontend formatted... ---

	@echo === formatting backend ===

	cd src-tauri && cargo fmt
	
	@echo --- backend formatted ---

	@echo --- format completed ---

.PHONY: dev
dev:
	@echo === starting development server... ===

	pnpm tauri dev --no-watch

	@echo --- development server exited ---

.PHONY: build
build:
	@echo === building application... ===

	pnpm tauri build

	@echo --- application build completed ---
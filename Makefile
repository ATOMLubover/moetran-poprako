fmt:
	@echo === formatting frontend... ===

	pnpm format

	@echo --- frontend formatted... ---

	@echo === formatting backend ===

	cd src-tauri && cargo fmt
	
	@echo --- backend formatted ---

	@echo --- format completed ---

dev:
	@echo === starting development server... ===

	pnpm tauri dev

	@echo --- development server exited ---
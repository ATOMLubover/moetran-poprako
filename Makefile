fmt:
	@echo === formatting frontend... ===

	pnpm format

	@echo --- frontend formatted... ---

	@echo === formatting backend ===

	cd src-tauri && cargo fmt
	
	@echo --- backend formatted ---

	@echo --- format completed ---
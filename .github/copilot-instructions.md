# Copilot Instructions for Moetran Native

## Architecture Overview

This is a Tauri desktop application with Vue 3 frontend and Rust backend. The app manages translation projects, teams, and users for the Moetran platform.

- **Frontend**: Vue 3 + TypeScript + Pinia stores, located in `src/`
- **Backend**: Rust with Tauri, handling IPC and external API calls in `src-tauri/`
- **Communication**: All backend interactions use Tauri's `invoke()` for IPC commands
- **Data Flow**: Frontend stores (Pinia) manage state; IPC calls fetch/update data from Rust backend, which proxies external APIs (Moetran, Poprako)

Key directories:

- `src/stores/`: Pinia stores for state management (e.g., `useRouterStore` for navigation)
- `src/ipc/`: Frontend IPC wrappers (e.g., `auth.ts` for token management)
- `src-tauri/src/`: Rust modules (e.g., `auth.rs`, `project.rs`) implementing IPC commands

## Developer Workflows

- **Development**: Run `pnpm tauri dev` to start dev server with hot reload
- **Build**: Use `pnpm tauri build` for production builds
- **Formatting**: `make fmt` formats both frontend (Prettier) and backend (cargo fmt)
- **Debugging**: Use VS Code with Tauri extension; logs via Rust's tracing (set `RUST_LOG=debug`)

No automated tests exist; validate manually via UI interactions.

## Project Conventions

- **State Management**: Use Pinia composition API with `defineStore()`; avoid direct mutations
- **IPC Calls**: Always wrap Tauri's `invoke()` in `src/ipc/` functions with typed interfaces
- **Navigation**: Use `useRouterStore` for view switching; pass params via store refs
- **Error Handling**: IPC functions return typed responses; handle errors in components via try/catch
- **File Structure**: Components in `src/components/`, views in `src/views/`, models in `src/api/model/`
- **Naming**: camelCase for files/components; PascalCase for Vue components

Example IPC usage:

```typescript
// In src/ipc/auth.ts
export async function getMoetranToken(): Promise<string | null> {
  return await invoke<string | null>('get_moetran_token');
}
```

Example store usage:

```typescript
// In src/stores/user.ts
export const useUserStore = defineStore('user', () => {
  const user = ref<ResUser | null>(null);
  // ...
});
```

## Code Style

- **Comments**: Use Chinese for all comments; memo-style, not tutorial-style.
- **Vue Frontend**:
  - Use `<script setup lang="ts">` with Composition API.
  - Comment core functions with `//` (multi-line ok).
  - Add blank lines between statements unless in single/double-line blocks.
  - Use semicolons; capitalize log messages without periods.
  - Strict TypeScript: no `any`, use interfaces.
  - High cohesion/low coupling; inject state from parent components.
  - Mock functions prefixed with `__mock`.
  - Template/style: Simple, modern, light colors; no scrollbars; icons over text.
- **Rust Backend**:
  - Add blank lines between statements in multi-line blocks.
  - Use `tracing::info!` for structured logs in `tauri::command` functions.
  - Group DTOs with related functions.
  - Use `tracing::debug!` for semantic debug logs in long functions.
  - Comments in Chinese, logs in English.

## Integration Points

- **External APIs**: Backend handles HTTP requests to Moetran/Poprako APIs; tokens stored securely via Tauri's storage
- **Cross-Component**: Use Pinia stores for shared state; emit events sparingly, prefer store updates
- **Dependencies**: Managed via pnpm; Rust crates in `src-tauri/Cargo.toml`

Focus on IPC for any new backend features; avoid direct HTTP from frontend.</content>
<parameter name="filePath">d:\my_projects\moetran-native\.github\copilot-instructions.md

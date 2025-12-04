# moetran-poprako

moetran-poprako is a native app based-on [moetran](https://moetran.com/).It provides user with enhanced project management capabilities and experience on translating or proofreading.

For server implementatios, see [moetran-poprako-support](https://github.com/ATOMLubover/moetran-poprako-support).

## Quick Start

### Prerequisites

- Node.js (LTS)
- pnpm
- Rust toolchain + cargo
- Tauri requirements for Windows (see Tauri docs)

### Local development

```powershell
pnpm install
make dev
```

### Format all files

```powershell
make fmt
```

### Build for production

```powershell
pnpm tauri build
```

## Acknowledgements

Special thanks to [moetran](https://moetran.com) for the original project inspiration and to the
[moeflow dev team (moeflow-com)](https://github.com/moeflow-com) for their open-source work and contributions.

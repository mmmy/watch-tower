# Watch Tower Native

This repository now ships a single active desktop shell:

- `native-shell/`: the Windows-native Slint shell

The old Tauri + React fallback has been removed from the main repo path.

## Default Windows Run Path

Use the default Slint shell from Windows PowerShell:

```powershell
npm run dev
```

This default path does not use WSL.

## Default Build And Check

```powershell
npm run check
npm run build
npm run test
```

These commands now target `native-shell/`.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

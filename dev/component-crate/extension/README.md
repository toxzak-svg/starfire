# Component Crate - VS Code Extension

## Quick Start

1. Open this folder in VS Code
2. Press `F5` to run the extension in a new window
3. Use the extension commands:

## Commands

| Command | Shortcut | Description |
|---------|----------|-------------|
| `Component Crate: Open` | - | Open the crate dashboard |
| `Component Crate: Search` | `⌘⇧C` | Quick search and insert |
| `Component Crate: Save Selection` | - | Save selected code to crate |

## Features

- **Sleek dark UI** - Electric lime accent, minimal design
- **Quick Search** - `⌘⇧C` to search and insert
- **Save Selection** - Select code → save with name + tags
- **Live Preview** - See component code before inserting
- **Local Storage** - Components stored in `~/.component-crate/crate.json`

## Usage

1. Select some code in your editor
2. `Cmd+Shift+P` → "Component Crate: Save Selection"
3. Enter name (e.g., "ButtonPrimary")
4. Enter tags (e.g., "button, form, primary")

Later:
- `⌘⇧C` → Search for component → Insert at cursor

## Install for Development

```bash
cd extension
npm install
npx tsc
```

Then press `F5` in VS Code to test.

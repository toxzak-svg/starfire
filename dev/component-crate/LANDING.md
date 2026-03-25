# Component Crate - Landing Page

## The Vibe

**Not generic SaaS.** This is a tool for builders, by builders. 
- Dark-first aesthetic (not another light-mode landing)
- Retro-terminal meets modern UI
- Show the actual product, not stock illustrations
- Typography: JetBrains Mono for headings, Inter for body
- Accent color: Electric lime (#BFFF00) or coral (#FF6B6B)

---

## Positioning

**VS Code Extension first.**
- Inline search → insert at cursor
- Import from your GitHub repos
- Web companion for management later

**The pitch:**
"We built this because our own snippet folders were a disaster."

---

## Page Structure

### 1. The Hook (Hero - VS Code Extension)

```
┌─────────────────────────────────────────────────────────────┐
│  COMPONENT CRATE                        [Get Extension]   │
│                                                             │
│   Your components in your editor.                          │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐  │
│   │  > Search components...                    ⌘⇧P     │  │
│   │                                                     │  │
│   │  ┌──────────────────────────────────────────────┐  │  │
│   │  │  ButtonPrimary    │  Modal      │  DataTable │  │  │
│   │  │  ButtonSecondary  │  Card       │  Form      │  │  │
│   │  │  Input            │  Sidebar    │  Navbar    │  │  │
│   │  └──────────────────────────────────────────────┘  │  │
│   │                        ↓ Insert at cursor           │  │
│   └─────────────────────────────────────────────────────┘  │
│                                                             │
│   Import from GitHub repos. Sync with your code.          │
└─────────────────────────────────────────────────────────────┘
```

**Copy:**
- "Your components. In your editor."
- "Stop hunting for that modal."
- "Code once. Insert forever."

---

### 2. The Problem

```
┌────────────────────────────────────────┐
│  Your folder of components is a mess  │
│                                        │
│  /components/
│  ├── Modal.tsx                        │
│  ├── ModalV2.tsx                      │
│  ├── modal_final.tsx                  │
│  ├── Table.tsx                        │
│  ├── DataTable.tsx                    │
│  └── TODO_cleanup.txt                  │
│                                        │
│  We know. We built this for ourselves.│
└────────────────────────────────────────┘
```

---

### 3. How It Works

```
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  ⌘⇧P Search  │  │   SELECT      │  │   PASTE      │
│              │  │              │  │              │
│  Find any    │  │  Pick from    │  │  Insert at   │
│  component   │  │  your crate   │  │  cursor      │
│              │  │              │  │              │
│  [search]    │  │  Modal ✓     │  │  📋 pasted   │
└──────────────┘  └──────────────┘  └──────────────┘
```

---

### 4. GitHub Import (Key Feature)

```
┌────────────────────────────────────────────────────────┐
│  IMPORT FROM GITHUB                                    │
│                                                        │
│  1. Connect your GitHub                              │
│  2. Browse your repos                                 │
│  3. Select files/folders                              │
│  4. Imported to your crate                            │
│                                                        │
│  [Connect GitHub]                                     │
│                                                        │
│  Optional: Keep synced with source                    │
└────────────────────────────────────────────────────────┘
```

---

### 5. The Limit (Freemium)

```
┌────────────────────────────────────────────────────────┐
│                    FREE TIER                           │
│                                                        │
│              10 components                             │
│                                                        │
│         [||||||||||] 0/10 used                        │
│                                                        │
│   $5/mo for 50 • $15/mo for 200 • $49/mo unlimited  │
└────────────────────────────────────────────────────────┘
```

---

### 6. Marketplace (Phase 2)

```
┌────────────────────────────────────────────────────────┐
│  FROM OTHER BUILDERS              [Coming Soon]        │
│                                                        │
│  Launching when we hit 1,000 crates.                  │
└────────────────────────────────────────────────────────┘
```

---

### 7. CTA

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│   Install for VS Code.                                │
│   10 free components await.                           │
│                                                        │
│   [Install Extension]                                 │
│   Free. No account required (for local).              │
│                                                        │
└────────────────────────────────────────────────────────┘
```

---

## Design Tokens

```css
:root {
  --bg-primary: #0D0D0D;
  --bg-secondary: #1A1A1A;
  --bg-tertiary: #262626;
  
  --text-primary: #FAFAFA;
  --text-secondary: #A3A3A3;
  --text-tertiary: #737373;
  
  --accent: #BFFF00;
  --accent-hover: #D4FF4D;
  --accent-muted: rgba(191, 255, 0, 0.1);
  
  --border: #333333;
  
  --font-mono: 'JetBrains Mono', monospace;
  --font-sans: 'Inter', sans-serif;
}
```

---

## Pages / Views

| View | Purpose |
|------|---------|
| VS Code Extension | Main product |
| Web Dashboard | Crate management, settings |
| Marketplace | Browse packs (v2) |

---

## Landing Page Copy (Full)

```
COMPONENT CRATE

Your components. In your editor.

⌘⇧P to search. One click to insert.

[Install Extension] [Connect GitHub]

---

Your folder of components is a mess.
Modal.tsx / ModalV2.tsx / modal_final.tsx
We know. We built this for ourselves.

---

How it works

1. ⌘⇧P to search
2. Select component
3. Insert at cursor

---

Import from GitHub

Connect your repos. Select files. Import to crate.
Optional sync with source.

---

Free: 10 components
$5/mo: 50 components  
$15/mo: 200 components
$49/mo: Unlimited

---

Start your crate.
10 free components await.

[Install for VS Code]
Free. No account required.
```

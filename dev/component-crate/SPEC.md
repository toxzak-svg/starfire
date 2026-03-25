# Component Crate - Specification

## The Idea

A VS Code extension for managing Tailwind + React components + optional GitHub import.

**Core value:** "I never have to hunt for that modal/table again."

**The Hook:** Free users get 10 components saved. Pay for more.

---

## Core Features

### Phase 1: VS Code Extension (Primary)

1. **Inline Search** — Cmd/Ctrl+Shift+P to search your crate
2. **Quick Insert** — Paste component at cursor
3. **Save Selection** — Select code → save to crate
4. **GitHub Import** — Import from connected repo (any file/folder)
5. **Live Preview** — Hover to preview component
6. **Limit System** — Free: 10 components, then pay

### Phase 2: Web Companion

- Full crate management (web dashboard)
- Settings, billing, account
- GitHub connection management (if not in VS Code)

### Phase 3: Marketplace (v2)

- After 1,000+ crates
- Buy/sell component packs

---

## GitHub Integration

- **Import from repo** — Pull any component/path from your GitHub repos
- **Sync** — Keep components synced with your source of truth (optional)
- **Partial imports** — Select specific files/folders from a repo
- **Auth:** GitHub OAuth (user grants repo access)
- **How it works:**
  1. Connect GitHub account
  2. Browse your repos in VS Code
  3. Select folder/file → Import to crate
  4. Optional: Mark as "synced" (pull updates later)

---

## The Limit (Freemium)

| Tier | Components | Price |
|------|-----------|-------|
| Free | 10 | $0 |
| Hobby | 50 | $5/mo |
| Pro | 200 | $15/mo |
| Team | Unlimited | $49/mo |

- GitHub imports don't count against limit
- Marketplace purchases don't count against limit

---

## User Flows

### Onboarding (VS Code)
1. Install extension from VS Code Marketplace
2. (Optional) Connect GitHub for import
3. Start saving components

### Saving a Component
1. Select code in editor → Right-click "Save to Crate"
2. Add name, tags, choose folder
3. Saved to local storage (or cloud if signed in)

### GitHub Import
1. Cmd/Ctrl+Shift+P → "Import from GitHub"
2. Browse repos → Select file/folder
3. Import to crate

---

## Technical Stack

### VS Code Extension
- **Framework:** TypeScript + VS Code API
- **Build:** VSCE (VS Code Extension Manager)
- **State:** Local JSON + optional cloud sync
- **GitHub API:** Octokit for repo access
- **Editor:** CodeMirror or Monaco (embedded for preview)

### Web Companion (Optional)
- **Frontend:** Next.js + React
- **Styling:** Tailwind
- **Auth:** GitHub OAuth
- **Database:** PostgreSQL (Supabase/Neon)

---

## Deployment

### Phase 1: VS Code Extension

| Resource | Service | Est. Cost |
|----------|---------|-----------|
| VS Code Marketplace | Free | $0 |
| Extension Hosting | GitHub Releases | $0 |

### Phase 2: Web + Cloud Sync
- Vercel (frontend)
- Supabase/Neon (database)

### Phase 3: Marketplace
- Stripe Connect

---

## Why This Works

1. **In-editor workflow** — Don't leave VS Code
2. **GitHub import** — Pull from existing repos
3. **Storage hook** — 10 free, then pay
4. **No lock-in** — Your code, copy-pasteable

---

## Names (Pick One)

- Component Crate (current)
- Crate
- Clip
- DevComponents

---

## Next Steps

- [ ] Initialize VS Code extension project
- [ ] Basic extension setup (command palette, storage)
- [ ] Search + insert functionality
- [ ] GitHub OAuth + repo browser
- [ ] Landing page for extension

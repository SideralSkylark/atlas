# Atlas — Project Specification

> A general-purpose Android Git client built with Tauri 2, Vue 3, and Rust.
> Open source. No accounts. No cloud. Just Git.

---

## Vision

Atlas is the Git client that doesn't exist on Android. It works with any remote —
GitHub, GitLab, Gitea, self-hosted — because it speaks Git directly. The other
devices in your workflow don't need Atlas. They just need Git.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | Vue 3 + TypeScript (Vite) |
| Backend | Rust |
| Framework | Tauri 2 |
| Git | `git2` crate (libgit2 bindings) |
| Syntax highlighting | `syntect` |
| Markdown rendering | `pulldown-cmark` |
| Distribution | F-Droid (primary), APK sideload |

---

## Milestones

### Phase 0 — Build Environment
- [x] Tauri 2 + Vue 3 scaffolded
- [x] App builds and runs on Android device
- [x] USB and wireless ADB working

### Phase 1 — Git Core (HTTPS)
- [x] Clone over HTTPS
- [x] List repos
- [x] Browse file tree
- [x] Pull (fetch + merge)
- [x] Push over HTTPS
- [x] PAT credential management

### Phase 2 — File Rendering
- [ ] Render HTML files in WebView
- [ ] Render Markdown
- [ ] Syntax-highlighted code viewer (syntect)
- [ ] Search across filenames

### Phase 3 — Auth
- [ ] SSH key generation
- [ ] SSH key import
- [ ] SSH clone/pull/push
- [ ] Secure credential storage (Android Keystore)

### Phase 4 — Git Advanced
- [ ] Branch list, create, switch
- [ ] Commit history viewer
- [ ] Staging area (pick files to commit)
- [ ] Diff viewer

### Phase 5 — Editor
- [ ] In-app text editor
- [ ] Commit from editor
- [ ] Push after commit

### Phase 6 — Polish & Release
- [ ] Everforest Hard Dark theme
- [ ] Proper mobile UI (touch targets, gestures)
- [ ] Performance pass
- [ ] F-Droid metadata and build recipe
- [ ] README and documentation

---

## Architecture

```
Android Device
└── Atlas (Tauri 2)
    ├── Vue 3 frontend (WebView)
    │   ├── Repo list screen
    │   ├── File browser screen
    │   ├── File viewer screen
    │   ├── Git operations screen
    │   └── Editor screen
    └── Rust backend
        ├── git2 (clone, pull, push, branch, diff, log)
        ├── syntect (syntax highlighting)
        ├── pulldown-cmark (markdown)
        └── Android Keystore (credentials)
```

---

## File Storage

Repos are stored in private app storage:
```
/data/data/com.skylark.atlas/files/repos/
└── repo-name/
    └── (git repo contents)
```

Private storage is intentional — no external storage permissions needed,
no risk of other apps modifying repo state.

---

## Constraints & Principles

- **Rust-first**: all business logic in Rust. Vue handles UI only.
- **No accounts**: Atlas never talks to any Atlas server. It only talks to Git remotes.
- **No database**: repo state lives in the Git repo itself.
- **Offline-first**: all operations except sync work without internet.
- **One learning track**: Rust is the focus. Vue is already known. No new JS frameworks.
- **Open source**: MIT or Apache 2.0 license. F-Droid compatible.


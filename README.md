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

| Layer               | Technology                      |
| ------------------- | ------------------------------- |
| Frontend            | Vue 3 + TypeScript (Vite)       |
| Backend             | Rust                            |
| Framework           | Tauri 2                         |
| Git                 | `git2` crate (libgit2 bindings) |
| Syntax highlighting | `syntect`                       |
| Markdown rendering  | `pulldown-cmark`                |
| Distribution        | F-Droid (primary), APK sideload |

---

## Milestones

### Phase 0 — Build Environment

* [x] Tauri 2 + Vue 3 scaffolded
* [x] App builds and runs on Android device
* [x] USB and wireless ADB working

### Phase 1 — Git Core (HTTPS)

* [x] Clone over HTTPS
* [x] List repos
* [x] Browse file tree
* [x] Pull (fetch + merge)
* [x] Push over HTTPS
* [x] PAT credential management

### Phase 2 — File Rendering

* [x] Render HTML files in WebView
* [x] Render Markdown
* [x] Syntax-highlighted code viewer (syntect)
* [x] Search across filenames

### Phase 3 — Security

* [x] Android Keystore integration
* [x] Secure PAT storage

### Phase 4 — Git Workflow

* [x] Branch list
* [x] Create branch
* [x] Switch branch
* [x] Commit history viewer
* [x] Staging area
* [x] Commit changes
* [x] Diff viewer

### Phase 5 — Editor

* [ ] add and delete files
* [ ] delete branches
* [x] In-app text editor
* [x] Save file changes
* [x] Commit from editor
* [x] Push after commit

### Phase 6 — SSH Support

* [ ] SSH key generation
* [ ] SSH key import
* [ ] SSH clone
* [ ] SSH pull
* [ ] SSH push
* [ ] Host key verification
* [ ] Known hosts management

### Phase 7 — Polish & Release

* [ ] Everforest Hard Dark theme
* [ ] Proper mobile UI (touch targets, gestures)
* [ ] Performance pass
* [ ] F-Droid metadata and build recipe
* [ ] README and documentation

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

* **Rust-first**: all business logic in Rust. Vue handles UI only.
* **No accounts**: Atlas never talks to any Atlas server. It only talks to Git remotes.
* **No database**: repo state lives in the Git repo itself.
* **Offline-first**: all operations except sync work without internet.
* **One learning track**: Rust is the focus. Vue is already known. No new JS frameworks.
* **Open source**: MIT or Apache 2.0 license. F-Droid compatible.

--- 

## Roadmap for frontend

### 1. Visual Polish & Identity
*   **Elevation & Depth**: Refine the "Everforest Hard Dark" theme with a consistent elevation system using subtle shadows and layered backgrounds (bg0 to bg3) to better distinguish between the background and interactive cards.
*   **Typography Hybrid**: Introduce a clean sans-serif font (e.g., *Inter* or *Geist*) for primary UI elements (labels, buttons, headers) to improve legibility on small screens, while maintaining monospace for code, hashes, and diffs.
*   **Micro-interactions**: Add subtle haptic feedback (via Tauri plugins) for critical actions like successful commits, cloning, or long-press reveals.

### 2. Navigation & Layout Evolution
*   **Bottom Navigation**: As features grow, transition to a bottom navigation bar for top-level views (e.g., Repositories, Activity, Settings) to provide better "thumb-reach" accessibility.
*   **Adaptive Views**: Implement a master-detail split-pane layout for tablets and foldable devices, allowing users to see the file tree and editor side-by-side.
*   **Sticky Context**: Implement shrinking headers in the file browser that provide more vertical space as the user scrolls, while keeping breadcrumbs visible.

### 3. Enhanced Git UX
*   **Hunk-Level Staging**: Allow staging or unstaging of specific hunks (blocks of code) directly from the Diff bottom sheet, rather than just whole files.
*   **Gesture Navigation**: Add "Swipe-to-Back" for file hierarchy navigation and "Pull-to-Refresh" for triggering `git fetch` or refreshing file lists.
*   **Optimistic UI**: Use optimistic updates for staging and branch switching to make the interface feel instantaneous while Git operations complete in the background.

### 4. Editor & Content Experience
*   **Mobile Editor Toolbar**: Add a specialized "symbol bar" above the soft keyboard in the Editor for common characters like `{`, `}`, `[`, `]`, and `tab` to reduce keyboard-switching fatigue.
*   **Rich Previews**: Add native-feeling image previews and interactive Markdown rendering with support for task lists and tables.
*   **Global Search Expansion**: Enhance the search interface to include "Search in File Content" (ripgrep-style) across the entire repository.

### 5. Accessibility & Customization
*   **Theme Engine**: Support for an "Everforest Light" mode and a "Pure Black" (OLED) theme for battery saving.
*   **Touch Target Audit**: Ensure all interactive elements meet the minimum 44x44dp touch target standard for improved accessibility.

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

* [x] add and delete files
* [x] delete branches
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

* [x] Everforest Hard Dark theme
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

## Todo

### High Priority
- [ ] **Conflict Resolution UI**: A dedicated view to handle merge conflicts within the app.
- [ ] **SSH Support**: Implementation of Phase 6 (Key generation, import, and SSH-based remotes).
- [x] **File Management**: Create, delete, and rename files/directories from the file browser.
- [ ] **Branch Management**: Delete branches (local and remote) and merge support.

### Git Enhancements
- [ ] **Commit Amending**: Support for `git commit --amend`.
- [ ] **Visual History Graph**: A canvas or SVG-based graph to visualize branch merges and history.
- [ ] **Blame View**: Show who last modified each line in the file viewer.

### Editor & Viewer
- [ ] **Search in Files**: Global "grep" functionality to find text across the entire repository.
- [ ] **Image & PDF Support**: View non-text assets directly in the file viewer.

### UI/UX & Polish
- [ ] **Custom Themes**: Support for different color schemes beyond Everforest.
- [ ] **App Shortcuts**: Long-press app icon shortcuts to jump to recent repositories.
- [ ] **Refined Gestures**: Enhanced swipe-to-action patterns for common git tasks.
- [ ] **Activity View**: Populate the dedicated Activity tab with a global feed of recent git events across all repos.

### Reliability
- [ ] **Automated Testing**: Unit tests for Rust git logic and E2E tests for the Vue frontend.
- [ ] **Error Boundaries**: Improved global error handling and user-friendly recovery paths.
- [ ] **Performance Pass**: Optimize large repository loading and diff rendering.

## UI/UX

This document outlines the architectural and stylistic requirements for the Atlas project, specifically focusing on cross-theme consistency and High-Fidelity
         Light Mode support.
       4
       5 ## 1. Depth & Shadows
       6 Shadows must remain effective and visible across both dark and light modes.
       7 - **Variable-Based Shadows:** Always use `var(--shadow-sm)`, `var(--shadow-md)`, and `var(--shadow-lg)`. Do not hardcode `rgba(0,0,0,0.3)` in components.
       8 - **Light Mode Intensity:** Light mode shadows should have a larger blur radius and lower opacity, but higher spread to maintain depth without feeling "dirty".
       9 - **Inner Depth:** Use `var(--shadow-inset)` (defined as `inset 0 1px 0 rgba(255,255,255,0.04)` in dark and a subtle dark inset in light) to give components a
         tactile, "carved" look.
      10
      11 ## 2. Component Structure
      12 - **Containers:** Components like `RepoItem` and `FileBrowser` entries should use `bg-bg1` as their primary background to contrast against the main `bg0`
         background.
      13 - **Borders:** Use `border-border` for structure. In Light Mode, `border-border` should be subtle but distinct.
      14 - **Interactive States:** Use `active:scale-[0.98]` and `transition-all` for tactile feedback on all buttons and clickable cards.
      15
      16 ## 3. Light Mode Color Strategy
      17 - **Avoid Pure White:** Light mode backgrounds (`--bg0`) should be slightly off-white (e.g., `#fdf6e3` for Everforest) to reduce eye strain.
      18 - **Contrast Ratios:** Ensure `--fg-dim` remains legible in light mode. It should be a softer version of the text, not an invisible one.
      19 - **Accents:** High-vibrancy colors like `yellow` and `green` may need slight adjustment in light mode to maintain punch without losing detail on light
         backgrounds.
      20
      21 ## 4. Anti-Patterns
      22 - **Hardcoded Colors:** Never use hex codes like `#272e33` inside `.vue` files. Use Tailwind classes (`bg-bg0`) or CSS variables (`var(--bg0)`).
      23 - **Hardcoded Opacity:** Be careful with `bg-bg1/80`. While it works for dark mode glassmorphism, it can wash out in light mode. Prefer solid colors or
         high-contrast transparency (`/90`).
      24 - **Fixed Shadows:** Do not use Tailwind's default `shadow-md` if it doesn't utilize the theme's custom variables, as it will look inconsistent between themes.
      25
      26 ## 5. Implementation Checklist
      27 - [ ] Shadows are visible in both Dark and Light modes.
      28 - [ ] Interactive elements provide tactile feedback (scaling/color change).
      29 - [ ] Text contrast meets accessibility standards in both modes.
      30 - [ ] Inner shadows (`shadow-inset`) are used to provide tactile depth.

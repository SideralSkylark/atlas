# UI/UX Styling Guidelines

This document outlines the architectural and stylistic requirements for the Atlas project, specifically focusing on cross-theme consistency and High-Fidelity Light Mode support.

## 1. Depth & Shadows
Shadows must remain effective and visible across both dark and light modes.
- **Variable-Based Shadows:** Always use `var(--shadow-sm)`, `var(--shadow-md)`, and `var(--shadow-lg)`. Do not hardcode `rgba(0,0,0,0.3)` in components.
- **Light Mode Intensity:** Light mode shadows should have a larger blur radius and lower opacity, but higher spread to maintain depth without feeling "dirty".
- **Inner Depth:** Use `var(--shadow-inset)` (defined as `inset 0 1px 0 rgba(255,255,255,0.04)` in dark and a subtle dark inset in light) to give components a tactile, "carved" look.

## 2. Component Structure
- **Containers:** Components like `RepoItem` and `FileBrowser` entries should use `bg-bg1` as their primary background to contrast against the main `bg0` background.
- **Borders:** Use `border-border` for structure. In Light Mode, `border-border` should be subtle but distinct.
- **Interactive States:** Use `active:scale-[0.98]` and `transition-all` for tactile feedback on all buttons and clickable cards.

## 3. Light Mode Color Strategy
- **Avoid Pure White:** Light mode backgrounds (`--bg0`) should be slightly off-white (e.g., `#fdf6e3` for Everforest) to reduce eye strain.
- **Contrast Ratios:** Ensure `--fg-dim` remains legible in light mode. It should be a softer version of the text, not an invisible one.
- **Accents:** High-vibrancy colors like `yellow` and `green` may need slight adjustment in light mode to maintain punch without losing detail on light backgrounds.

## 4. Anti-Patterns
- **Hardcoded Colors:** Never use hex codes like `#272e33` inside `.vue` files. Use Tailwind classes (`bg-bg0`) or CSS variables (`var(--bg0)`).
- **Hardcoded Opacity:** Be careful with `bg-bg1/80`. While it works for dark mode glassmorphism, it can wash out in light mode. Prefer solid colors or high-contrast transparency (`/90`).
- **Fixed Shadows:** Do not use Tailwind's default `shadow-md` if it doesn't utilize the theme's custom variables, as it will look inconsistent between themes.

## 5. Implementation Checklist
- [ ] Shadows are visible in both Dark and Light modes.
- [ ] Interactive elements provide tactile feedback (scaling/color change).
- [ ] Text contrast meets accessibility standards in both modes.
- [ ] Inner shadows (`shadow-inset`) are used to provide tactile depth.

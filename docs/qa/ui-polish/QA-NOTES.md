# UI polish QA notes

**Canon:** [`../../SLOPOS-I.md`](../../SLOPOS-I.md) §5  
**Do not** treat this file as a second design doc — only evidence pointers.

**Updated:** 2026-07-31 evening (post System7 chrome pass + SLOPOS-I rename).

## Latest capture set

| File | Theme | Notes |
|------|-------|-------|
| `01-desktop.png` | classic/light | Menu “SLOPOS”, “SLOPOS HD”, bevels, fixed icons |
| `02-desktop-dark.png` | dark / Graphite | Fuller-stack chrome; still not kit-parity |
| `03-spotlight.png` | classic | Overlay paints; also covered in `../v0.2.0/` |

Update screenshots in-place when iterating; keep the gap list in `SLOPOS-I.md` §5 only.

## Capture reminder (UTM)

Use sway headless + grim + `SLOPOS_LAYER_SHELL_CHROME=1` + software GL.
Reject blank/tiny PNGs. Recipe: [SLOPOS-I.md](../../SLOPOS-I.md) §7.

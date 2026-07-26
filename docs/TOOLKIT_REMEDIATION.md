# retro-kit remediation plan

**Status date:** 2026-07-26
**Why this document exists:** the apps look and behave fine on screen, so the
toolkit appears healthy. It is not. Both things are true at once, and this
explains why, then lays out the fix.

---

## 1. The root cause, in one paragraph

`retro-kit` widgets are **layout + data containers that do not handle input, and
mostly do not paint themselves**. Painting is done by a single 1200-line
downcast chain in `retro-sdk` (`draw_widget()` → `if let Some(b) =
w.as_any().downcast_ref::<Button>()`), and hit-testing is done **by each
application separately** (`if self.install_button.rect().contains(point) { … }`
in `apps/appstore/src/main.rs`). So the widget tree is real, the rects are real,
the pixels are real — but the `Widget` trait's own `draw()` and `handle_event()`
are dead weight. Every app re-implements interaction from scratch, and anything
that isn't hand-wired in an app simply doesn't respond.

### Evidence

`crates/retro-kit/src/button.rs` is the whole story in 40 lines:

```rust
fn draw(&self, theme: &ThemeContext) {
    let _bg = if self.state.hovered { … } else { … };   // computed
    let _text_color = theme.color(ThemeToken::ButtonText); // computed
}                                                        // …and discarded
```

```rust
fn handle_event(&mut self, event: &Event) -> EventResult {
    match event {
        Event::MouseDown { button: MouseButton::Left, .. } => {  // `..` = the
            self.state.hovered = true;                           // point is
            EventResult::Handled                                 // ignored
        }
        …
    }
}
```

Three defects in one widget: `draw()` paints nothing; `MouseDown` is consumed
**without checking the click is inside the button**; and there is no
`on_click` — a `Button` has no way to notify anyone it was pressed.

The infrastructure to fix this already exists and is simply unused:

```rust
// crates/retro-kit/src/widget.rs
pub struct WidgetState {
    pub id: WidgetId,
    pub rect: Rect,
    pub focused: bool,   // <- nothing ever sets this
    pub hovered: bool,
    …
}
pub trait Widget: Send {
    fn children(&self) -> Vec<&dyn Widget> { vec![] }          // <- tree exists
    fn children_mut(&mut self) -> Vec<&mut dyn Widget> { vec![] }
    fn handle_event(&mut self, _e: &Event) -> EventResult { EventResult::Ignored }
}
```

**This is why the fix is additive rather than a rewrite.** `WidgetId`,
`WidgetState.focused`, the children tree, and `EventResult` are all the right
primitives. Nothing drives them.

---

## 2. Target API

### 2.1 Hit-test dispatch — `crates/retro-kit/src/dispatch.rs` (new)

```rust
/// Deepest visible, enabled widget whose rect contains `at`.
pub fn widget_at(root: &dyn Widget, at: Point) -> Option<WidgetId>;

/// Deliver a pointer event to the deepest hit widget, bubbling toward the root
/// while handlers return `EventResult::Ignored`.
pub fn dispatch_pointer(root: &mut dyn Widget, at: Point, ev: &Event) -> EventResult;

/// Drive MouseEnter/MouseLeave from pointer motion, so `hovered` becomes real.
pub fn dispatch_motion(root: &mut dyn Widget, at: Point, hover: &mut Option<WidgetId>);
```

Depth-first, children painted last (topmost) tested first. `Visibility::Hidden`
and `enabled == false` subtrees are skipped.

### 2.2 Focus — `crates/retro-kit/src/focus.rs` (new)

```rust
pub struct FocusManager { focused: Option<WidgetId> }

impl FocusManager {
    pub fn focused(&self) -> Option<WidgetId>;
    pub fn focus(&mut self, root: &mut dyn Widget, id: WidgetId);
    pub fn clear(&mut self, root: &mut dyn Widget);
    /// Tab / Shift+Tab across the focusable widgets in tree order.
    pub fn focus_next(&mut self, root: &mut dyn Widget);
    pub fn focus_prev(&mut self, root: &mut dyn Widget);
    /// Route a key event to the focused widget; returns Ignored if none.
    pub fn dispatch_key(&mut self, root: &mut dyn Widget, ev: &Event) -> EventResult;
}
```

with one new trait method:

```rust
pub trait Widget: Send {
    /// Can this widget take keyboard focus? Default false.
    fn focusable(&self) -> bool { false }
}
```

`focus()` sets `WidgetState.focused` on the target and clears it everywhere
else — that single invariant fixes the "two `TextField`s both eat every
keystroke" bug, because `TextField::handle_event` will gate on
`self.state.focused`.

### 2.3 Activation callbacks

```rust
pub struct Button {
    state: WidgetState,
    label: String,
    pressed: bool,                                   // new: press vs hover
    on_click: Option<Box<dyn FnMut() + Send>>,
}

impl Button {
    pub fn on_click(mut self, f: impl FnMut() + Send + 'static) -> Self;
}
```

Correct semantics: `MouseDown` inside → `pressed = true`; `MouseUp` inside while
`pressed` → fire `on_click`; `MouseUp` outside or `MouseLeave` → cancel. Today
there is no press state and no release handling at all.

For apps that prefer polling over closures (which is how the current code is
written), also expose:

```rust
impl Button { pub fn take_clicked(&mut self) -> bool; }
```

so an app's `update()` can drain activations without restructuring around
callbacks. **This is the migration lever** — it lets apps adopt real dispatch
without being rewritten.

---

## 3. Per-widget work

| Widget | Verdict today | Change needed |
|---|---|---|
| `Button` | render-only; consumes clicks with no hit test; no activation | press/release state, `on_click` / `take_clicked`, real `draw()` |
| `TextField` | input works, but no focus gate, so all fields consume all keys | gate on `state.focused`, `focusable() = true`, caret + selection rendering, password masking (`is_password` is currently ignored when drawing) |
| `PopupButton` | toggles `open` on any click anywhere; the open menu is never rendered or selectable | hit-test the button rect, render the menu, hit-test items, `on_select(index)` |
| `TabView` | headers drawn but not clickable; `select_tab` has no callers | header rect hit-test → `select_tab`, keyboard Left/Right when focused |
| `ScrollView` | wheel events change an offset that is only applied inside `layout()`, which does not re-run → visually inert; offset unclamped | apply offset at paint time, clamp to content bounds, scrollbar hit-test/drag |
| `Dialog` | buttons unclickable | route through generic dispatch; default/cancel button semantics (Enter/Escape) |
| `ListView` | selection works via app-level hit tests | own the hit test; keyboard Up/Down/Home/End; `on_activate` for double-click |
| `TreeView` | same as ListView, plus expand/collapse | disclosure-triangle hit region, Left/Right to collapse/expand |
| `Slider` | drag math is correct; app drives it | own the drag grab, keyboard arrows, `on_change` |
| `MenuBar` | works (shell drives it) | keep, but move hit-testing into the widget |
| `Toolbar` | app-driven | own hit test, `on_action(index)` |
| `IconView` | app-driven selection and drag | own hit test + rubber-band selection; keyboard arrows |
| `SplitView` | divider not draggable | divider grab + min/max constraints |
| `StatusBar`, `Label`, `ProgressBar`, `MonospaceView` | display-only, which is correct | give them real `draw()` when the painter moves (§4) |
| `DockView`, `WorkspaceGridView` | shell-driven | own hit test, expose activation |

---

## 4. Migration order

Each step leaves the tree working and testable.

1. **Add `dispatch.rs` + `focus.rs`** with unit tests over synthetic widget
   trees. Nothing calls them yet. No behaviour change.
2. **Add `focusable()`, press state, and `take_clicked()`** to widgets. Still no
   behaviour change — apps keep their own hit tests.
3. **Port Settings** (the most widget-dense app) to generic dispatch: replace
   its hand-rolled `rect().contains(point)` chain with
   `dispatch_pointer(&mut self.root, point, ev)` and `take_clicked()`. Verify in
   the VM.
4. **Port the other four apps** one at a time.
5. **Port the shell** (`ShellDesktop::handle_event`), which is the biggest
   consumer and also owns the lock screen and menu bar.
6. **Delete app-level hit-testing.** Grep for `rect().contains(` — it should
   return nothing outside the toolkit.
7. **Move painting into widgets** (`draw(&self, canvas: &mut Canvas, theme:
   &ThemeContext)`) and delete the `draw_widget` downcast chain in `retro-sdk`.
   Do this last: it touches every widget and is the easiest to get wrong.

### Exit criteria

- `Settings` is fully operable with Tab / Shift+Tab / Enter / Space and no mouse.
- Two `TextField`s in one window route keystrokes only to the focused one.
- No `rect().contains(` outside `crates/retro-kit`.
- Clicking outside a `Button` does not consume the event.
- A widget added to an app renders without editing `retro-sdk`.

---

## 5. Estimated effort

| Step | Effort |
|---|---|
| 1–2 (dispatch, focus, activation primitives) | ~1 week |
| 3–4 (port five apps) | ~2 weeks |
| 5 (port the shell) | ~1 week |
| 6 (cleanup) | ~2 days |
| 7 (painter migration) | ~2 weeks |

Roughly **6–7 weeks of focused work**, and it unblocks essentially all app-level
improvement in `docs/ROADMAP.md` Phase 5.

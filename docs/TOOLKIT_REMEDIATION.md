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

And the app-side bypass, for every app:
`apps/finder/src/main.rs:409-429` (toolbar by index), `apps/settings/src/main.rs:974-996`,
`apps/textedit/src/main.rs:778`, `apps/appstore/src/main.rs:1236`.

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

Verdicts below are from a full read of all 21 widgets against their SDK painters
and their actual app consumers. **Three widgets are genuinely fine** — the
toolkit is not uniformly broken, which matters for planning.

| Widget | Verdict | Evidence / change needed |
|---|---|---|
| `ListView` | **works** | Real: guards `if !self.rect().contains(*point) { return Ignored }`, row pitch matches the painter. Add keyboard nav, scrolling, and implement the declared-but-unused `multi_select`. |
| `Slider` | **works** | Real end-to-end, rect-guarded drag with the same 9px inset as the painter, unit-tested. Add keyboard arrows and `on_change` (consumers poll `.value` today). |
| `MenuBar` | **works** | The one widget that publishes its own geometry (`menu_rects`, `dropdown_rect`) and hit-tests properly. Use it as the model for the others. |
| `Button` | render-only | `draw()` discards its computed colors; `MouseDown` returns `Handled` for **any** click in the window. Needs rect gate, press/release state, `on_click`/`take_clicked`. |
| `Toolbar` | render-only, **actively harmful** | Fans events to children in reverse with no rect check; combined with `Button` returning `Handled` unconditionally, **the last toolbar item swallows every left click in the window**. Both consumers therefore avoid it entirely. |
| `TextField` | partial | Input mutates text correctly, but `on_change` is never assigned anywhere in the repo, and there is no focus gate. Needs click-to-focus, `focusable()`, caret rendering, and password masking (`is_password` is ignored by the painter). |
| `IconView` | partial | Selection genuinely hit-tests item rects. But `on_double_click` is never assigned, so Finder and the shell both re-scan items themselves. Needs a real activation channel (`take_activated()`). |
| `TreeView` | partial, **wrong** | Hit-tests only the outer rect, then slices the view into three fixed percentage bands (`<30%`, `<60%`, else) returning hardcoded paths `[0,3]`/`[0,4]`/`[0,5]`. Clicking "Favorites" selects Desktop. Needs a real visible-row list built in `layout()` and shared with the painter. |
| `Window` | partial | `FocusIn`/`FocusOut` are winit *platform-window* events, not widget focus. The titlebar it appears to own is reimplemented in the shell with duplicated magic numbers. Should expose titlebar/close/zoom/resize rects from `layout()`. |
| `PopupButton` | dead | Any click anywhere toggles `open`; the painter never reads `open` or the non-selected items, so an open popup renders identically to a closed one. No click→item mapping exists. Not instantiated anywhere. |
| `TabView` | dead | Headers are painted with geometry computed inline in the painter; `TabView` stores none, so headers cannot be hit-tested and `select_tab` has no callers. Also launders lifetimes with `unsafe { &mut *(r as *mut dyn Widget) }` in `children_mut` — **fix or remove that regardless**. |
| `ScrollView` | dead | The SDK sends `MouseWheel` to the root with no pointer target, so the first `ScrollView` in the tree consumes every wheel event wherever the cursor is. Offset is never clamped to content height and no scrollbar is painted. |
| `SplitView` | dead | Divider is inert — `divider_position` is never modified by any event. Not used anywhere; Finder and Settings hand-roll sidebars in `layout()`. |
| `Dialog` | dead | Does not override `handle_event` at all, inheriting the trait default `Ignored`. No default button, no dismissal, no result. Not constructed anywhere. |
| `Label`, `ProgressBar`, `StatusBar`, `MonospaceView`, `DockView`, `WorkspaceGridView` | render-only *by design* | Correct as display-only; they just need real `draw()` when the painter moves (§4). |

### The one fix that unlocks the most

`Toolbar`, `Layout`, `SplitView` and `Window` all forward positional events to
children **without a rect check**. That single missing guard, plus `Button`
returning `Handled` unconditionally, is why every app abandoned the toolkit and
hand-rolled its own dispatch. One shared helper —

```rust
fn dispatch_positional(children: &mut [&mut dyn Widget], at: Point, ev: &Event) -> EventResult;
```

— used by all four containers is the highest-leverage change in this document.

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

# Animated Desktop Backgrounds (planned)

> Feature backlog item #3. Support for animated wallpapers (GIF/video/shader).

## Current state

- **Desktop rendering:** `ShellDesktop::render_background()` paints a static image
- **Wallpaper source:** `load_wallpaper_image()` reads PNG/JPG from disk
- **Layer:** background layer surface (Wayland layer-shell, exclusive_zone -1)

## Goals

1. **GIF animation** — play animated GIF on desktop
2. **Video wallpapers** — play short video loops (MP4, WebM, etc.)
3. **Shader wallpapers** — optional: procedural / realtime-rendered backgrounds
4. **Performance** — no frame drops, don't starve app rendering
5. **Configuration** — user can select animated wallpaper from Settings

## Design

### Types of animated wallpapers

#### GIF (easiest)
- Decode GIF frames + timing
- Play loop, sync to display (60 Hz)
- Low CPU cost

#### Video (medium)
- Use ffmpeg or gstreamer to decode
- Extract frame rate + timing
- Memory: buffer a few frames

#### Shader (hard, future)
- GLSL fragment shader running on GPU
- Procedural noise / animation
- Lowest CPU cost, highest visual impact

### Architecture

#### Layer 1: Wallpaper decoder (`retro-shell/src/wallpaper.rs` — new)
```rust
pub enum WallpaperSource {
    Static(Image),
    Gif(GifDecoder),
    Video(VideoDecoder),
    Shader(String), // GLSL source
}

pub struct Wallpaper {
    source: WallpaperSource,
    current_frame: Image,
    duration_ms: u32,
    elapsed_ms: u32,
}

impl Wallpaper {
    pub fn new(path: &Path) -> Result<Self>;
    pub fn update(&mut self, delta_ms: u32);
    pub fn frame(&self) -> &Image;
}
```

#### Layer 2: Shell integration
- `ShellDesktop` keeps a `Wallpaper` instance
- Update on each tick (`UiRuntime::tick()`)
- Pass current frame to background layer rendering

#### Layer 3: Settings UI
- Settings app shows wallpaper browser
- User selects from `~/.local/share/wallpapers/` or `/usr/share/wallpapers/`
- Config stored in `~/.config/retroshell/wallpaper.toml`

### Rendering

Current path (static):
```
ShellDesktop::render_background()
  → load_wallpaper_image()
  → canvas.draw_image(wallpaper, rect)
```

New path (animated):
```
ShellDesktop::update(delta_ms):
  wallpaper.update(delta_ms)

ShellDesktop::render_background():
  canvas.draw_image(wallpaper.frame(), rect)
```

### Performance considerations

- **60 Hz expectation:** desktop + apps both render at display refresh
- **Decoding:** background thread (rayon) for video frame decoding
- **Memory:** frame buffer limited (2-3 frames max for video)
- **Fallback:** if video codec unavailable, show static image

## Implementation phases

### Phase 1: GIF support
- Add `gif` crate dep
- Implement `GifDecoder`
- Update `ShellDesktop::update()` to step frames
- Test: Settings app shows "Wallpaper" picker (stub)

### Phase 2: Video support (optional, Phase 1.5)
- Add video decoder (ffmpeg-sys or gstreamer bindings)
- Threaded decoding + frame buffer

### Phase 3: Shader support (future)
- WGPU compute shader or fragment shader
- Procedural generation + animation

### Phase 4: Settings UI
- Browser for wallpapers directory
- Preview before apply
- Persistence + theme coordination

## Acceptance criteria

**Phase 1 (GIF):**
```bash
# Place an animated GIF in ~/Pictures/wallpaper.gif
# Settings → Desktop → Wallpaper → select it
# Desktop animates smoothly, no frame drops
```

**Phase 2 (video):**
```bash
# Same as GIF, but with ~/Pictures/wallpaper.mp4
# Video plays on loop, in sync with display refresh
```

**Phase 3 (shader):**
```bash
# Procedural animated background
# GPU-driven, minimal CPU usage
```

## Dependencies

- **retro-shell/src/wallpaper.rs** (new module)
- **GIF decoder crate** (gif or image crate's gif feature)
- **Video decoder** (ffmpeg-sys or gstreamer; check for OS availability)
- **Settings app** needs wallpaper browser UI

## Known risks

- **Codec availability:** video decoding depends on ffmpeg/gstreamer being installed
- **Performance regression:** if decoding blocks, desktop lag is noticeable
- **Memory:** buffering video frames could consume significant RAM on older systems
- **Licensing:** GPL codecs (ffmpeg) vs. proprietary (h.264)

## Timeline estimate
- **Phase 1 (GIF):** 2-3 days
- **Phase 2 (video):** 3-5 days (codec selection + threading)
- **Phase 3 (shader):** 3-5 days (WGPU setup + GLSL)
- **Phase 4 (Settings UI):** 2-3 days

**Total:** 2-3 weeks for full support (MVP without Phase 3).

## Notes

This feature is purely visual and can be deferred. It becomes most interesting once
the layer-shell chrome rework (done in Stage 2b) is proven stable — animated
backgrounds rely on the background layer being a real owned surface that the shell
controls, not a workaround in the app window.

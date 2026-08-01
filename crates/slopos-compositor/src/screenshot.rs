//! SIGUSR1-triggered framebuffer screenshot for QA.
//!
//! There is no host-side screenshot on the UTM/QEMU target (unlike VirtualBox's
//! `VBoxManage screenshotpng`), and reading the DRM scanout from another process
//! is not possible while the compositor holds it. So we capture from *inside* the
//! compositor: on `SIGUSR1`, render the current frame's elements into an offscreen
//! GL buffer, read the pixels back with `ExportMem`, and write a PNG. This works
//! even in the unprivileged (no DRM-master) path, since it never touches scanout.

use std::sync::atomic::{AtomicBool, Ordering};

use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::gles::{GlesRenderbuffer, GlesRenderer};
use smithay::backend::renderer::utils::draw_render_elements;
use smithay::backend::renderer::{
    Bind, Color32F, ExportMem, Frame, Offscreen, Renderer, TextureMapping,
};
use smithay::utils::{Buffer as BufferCoord, Physical, Rectangle, Size, Transform};

/// Set by the SIGUSR1 handler; consumed on the next frame.
pub static SHOT_REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn on_sigusr1(_sig: libc::c_int) {
    SHOT_REQUESTED.store(true, Ordering::SeqCst);
}

/// Install a SIGUSR1 handler that requests a screenshot on the next frame.
pub fn install_signal_handler() {
    unsafe {
        libc::signal(libc::SIGUSR1, on_sigusr1 as *const () as usize);
    }
}

fn shot_path() -> String {
    std::env::var("SLOPOS_SHOT_PATH").unwrap_or_else(|_| "/tmp/slopos-i-shot.png".into())
}

/// If a screenshot was requested via SIGUSR1, capture the current frame to PNG.
/// Best-effort: logs the outcome and always clears the request flag.
pub fn capture_if_requested(
    renderer: &mut GlesRenderer,
    elements: &[WaylandSurfaceRenderElement<GlesRenderer>],
    size: (i32, i32),
    clear: [f32; 4],
) {
    if !SHOT_REQUESTED.swap(false, Ordering::SeqCst) {
        return;
    }
    match capture(renderer, elements, size, clear) {
        Ok(path) => {
            tracing::info!(path = %path, "screenshot written");
            eprintln!("[slopos-compositor] screenshot written: {path}");
        }
        Err(e) => {
            tracing::warn!(error = %e, "screenshot failed");
            eprintln!("[slopos-compositor] screenshot failed: {e:#}");
        }
    }
}

fn capture(
    renderer: &mut GlesRenderer,
    elements: &[WaylandSurfaceRenderElement<GlesRenderer>],
    (w, h): (i32, i32),
    clear: [f32; 4],
) -> anyhow::Result<String> {
    let phys: Size<i32, Physical> = Size::from((w, h));
    let buf: Size<i32, BufferCoord> = Size::from((w, h));

    // create_buffer requires a fourcc that maps to GL internal RGBA8; Abgr8888
    // does, Argb8888 does not ("Unsupported pixel layout").
    let mut target: GlesRenderbuffer =
        Offscreen::<GlesRenderbuffer>::create_buffer(renderer, Fourcc::Abgr8888, buf)
            .map_err(|e| anyhow::anyhow!("create_buffer: {e}"))?;
    let mut fb = renderer
        .bind(&mut target)
        .map_err(|e| anyhow::anyhow!("bind: {e}"))?;

    let damage = [Rectangle::from_size(phys)];
    {
        let mut frame = renderer
            .render(&mut fb, phys, Transform::Normal)
            .map_err(|e| anyhow::anyhow!("render: {e}"))?;
        frame
            .clear(Color32F::from(clear), &damage)
            .map_err(|e| anyhow::anyhow!("clear: {e}"))?;
        draw_render_elements::<GlesRenderer, _, _>(&mut frame, 1.0, elements, &damage)
            .map_err(|e| anyhow::anyhow!("draw: {e}"))?;
        let _ = frame.finish().map_err(|e| anyhow::anyhow!("finish: {e}"))?;
    }

    // Read back as Argb8888 (GL_BGRA_EXT), the format mesa actually accepts for
    // glReadPixels here (plain GL_RGBA/Abgr8888 is rejected on this driver).
    let mapping = renderer
        .copy_framebuffer(&fb, Rectangle::from_size(buf), Fourcc::Argb8888)
        .map_err(|e| anyhow::anyhow!("copy_framebuffer: {e}"))?;
    let flipped = mapping.flipped();
    let pixels = renderer
        .map_texture(&mapping)
        .map_err(|e| anyhow::anyhow!("map_texture: {e}"))?;

    // Fourcc::Argb8888 is 0xAARRGGBB little-endian → memory bytes [B, G, R, A].
    // `image::RgbaImage` wants [R, G, B, A], so swap B<->R.
    let mut rgba = pixels.to_vec();
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    if flipped {
        let stride = (w as usize) * 4;
        let mut out = vec![0u8; rgba.len()];
        for row in 0..h as usize {
            let src = row * stride;
            let dst = (h as usize - 1 - row) * stride;
            out[dst..dst + stride].copy_from_slice(&rgba[src..src + stride]);
        }
        rgba = out;
    }

    let path = shot_path();
    let img = image::RgbaImage::from_raw(w as u32, h as u32, rgba)
        .ok_or_else(|| anyhow::anyhow!("RgbaImage::from_raw: buffer size mismatch"))?;
    img.save(&path)
        .map_err(|e| anyhow::anyhow!("save {path}: {e}"))?;
    Ok(path)
}

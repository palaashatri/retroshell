use retro_shell::RetroShell;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting RetroShell...");

    // Best-effort AT-SPI2 registration with structural shell chrome tree
    // (menu bar → desktop icons → dock + window). Connection is retained for
    // best-effort Focus/Object event emission from shell Tab chrome focus.
    // Still Orca-incomplete: no Text/Component interfaces, live tree sync, or
    // real DoAction routing. D-Bus events fail open when registry/bus absent;
    // in-process AccessibilityEventBus always works.
    match retro_kit::register_at_spi_shell_chrome("RetroShell") {
        Ok(()) => {
            if retro_kit::at_spi_registration_info().is_some() {
                tracing::info!(
                    "AT-SPI2 accessibility registration active (shell chrome tree; event emit best-effort)"
                );
            } else {
                tracing::info!(
                    "AT-SPI2 skipped (no session bus or registry); in-process a11y events only"
                );
            }
        }
        Err(err) => tracing::warn!("AT-SPI2 registration failed: {err}"),
    }

    let shell = match RetroShell::startup() {
        Ok(shell) => shell,
        Err(e) => {
            tracing::error!("Failed to start RetroShell: {}", e);
            return;
        }
    };

    tracing::info!("RetroShell initialized successfully");
    tracing::info!("Theme: {}", shell.theme_manager.read().current);
    tracing::info!(
        "Applications found: {}",
        shell.launch_services.read().bundles.len()
    );
    tracing::info!("Workspaces: {}", shell.workspace_manager.read().total);

    if let Err(e) = shell.run() {
        tracing::error!("Shell run error: {}", e);
    }
}

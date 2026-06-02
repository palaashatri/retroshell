use retro_shell::RetroShell;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting RetroShell...");

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

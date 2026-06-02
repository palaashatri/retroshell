use retro_kit::window::Window;
use retro_kit::menu::{Menu, MenuItem};
use retro_bus::RetroBus;

pub struct Application {
    pub name: String,
    pub bundle_id: String,
    pub main_window: Option<Window>,
    pub menus: Vec<Menu>,
    pub bus: Option<RetroBus>,
    pub running: bool,
}

impl Application {
    pub fn new(name: &str, bundle_id: &str) -> Self {
        Self {
            name: name.to_string(),
            bundle_id: bundle_id.to_string(),
            main_window: None,
            menus: vec![],
            bus: None,
            running: false,
        }
    }

    pub fn with_bus(mut self, bus: RetroBus) -> Self {
        self.bus = Some(bus);
        self
    }

    pub fn set_main_window(&mut self, window: Window) {
        self.main_window = Some(window);
    }

    pub fn set_menus(&mut self, menus: Vec<Menu>) {
        self.menus = menus;
    }

    pub fn run(&mut self) {
        self.running = true;
        tracing::info!("Application '{}' started", self.name);
    }

    pub fn quit(&mut self) {
        self.running = false;
        tracing::info!("Application '{}' quit", self.name);
    }
}

pub trait AppDelegate {
    fn app_did_finish_launching(&mut self);
    fn app_will_terminate(&mut self);
    fn app_did_resign_active(&mut self);
    fn app_did_become_active(&mut self);
}

pub fn build_menu(title: &str) -> Menu {
    Menu::new(title)
}

pub fn menu_item(label: &str, action: &str) -> MenuItem {
    let mut item = MenuItem::action(label);
    item.with_action(action);
    item
}

pub fn separator() -> MenuItem {
    MenuItem::separator()
}

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

pub struct RetroEventLoop {
    pub event_loop: EventLoop<()>,
}

impl Default for RetroEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl RetroEventLoop {
    pub fn new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        Self { event_loop }
    }
}

pub trait RetroAppHandler {
    fn init(&mut self, event_loop: &ActiveEventLoop);
    fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent);
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

struct AppHandlerWrapper<'a, H: RetroAppHandler> {
    handler: &'a mut H,
}

impl<'a, H: RetroAppHandler> ApplicationHandler for AppHandlerWrapper<'a, H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.handler.init(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handler.handle_window_event(event_loop, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.handler.about_to_wait(event_loop);
    }
}

impl RetroEventLoop {
    pub fn run<H: RetroAppHandler>(
        self,
        handler: &mut H,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = self.event_loop;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut wrapper = AppHandlerWrapper { handler };
        event_loop.run_app(&mut wrapper)?;
        Ok(())
    }
}

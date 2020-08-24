use winit::event_loop::EventLoop;
use crate::error::GameResult;

pub trait RenderBackend {
    fn create_window(&self) -> GameResult;
}

impl RenderBackend {
    pub fn new(event_loop: &EventLoop<()>) -> Box<dyn RenderBackend> {
        Box::new(RgxRenderBackend::new())
    }
}

pub struct RgxRenderBackend {}

impl RenderBackend for RgxRenderBackend {
    fn create_window(&self) -> GameResult<()> {
        Ok(())
    }
}

impl RgxRenderBackend {
    pub fn new() -> RgxRenderBackend {
        Self {}
    }
}

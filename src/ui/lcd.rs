use std::sync::Arc;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use winit::dpi::LogicalSize;

pub const WIDTH: u32 = 160;
pub const HEIGHT: u32 = 144;
pub const SCALING_FACTOR: u32 = 2;

pub struct LCD {
    pixels: Option<Pixels<'static>>,
    window: Option<Arc<Window>>,
    pixel_buffer: [u8; (WIDTH * SCALING_FACTOR * HEIGHT * SCALING_FACTOR * 4) as usize],
}

impl Default for LCD {
    fn default() -> Self {
        let pixel_buffer = [0xFF; (WIDTH * SCALING_FACTOR * HEIGHT * SCALING_FACTOR * 4) as usize];
        Self {
            pixels: None, window: None, pixel_buffer
        }
    }
}

impl ApplicationHandler for LCD {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Rainier")
            .with_resizable(false)
            .with_inner_size(LogicalSize::new(WIDTH * SCALING_FACTOR, HEIGHT * SCALING_FACTOR));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let size = window.inner_size();

        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());

        let pixels: Pixels<'static> = Pixels::new(WIDTH * SCALING_FACTOR, HEIGHT * SCALING_FACTOR, surface_texture).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);

        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let pixels = self.pixels.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let frame = pixels.frame_mut();
                frame.copy_from_slice(&self.pixel_buffer);

                if pixels.render().is_err() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                pixels.resize_surface(size.width, size.height).unwrap();
            }
            _ => {}
        }
    }
}

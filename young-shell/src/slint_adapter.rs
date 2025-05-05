use std::cell::Cell;
use std::rc::Rc;

use slint::{
    platform::{
        software_renderer::{
            RepaintBufferType::{self},
            SoftwareRenderer,
        },
        Platform, WindowAdapter,
    },
    PhysicalSize, Rgb8Pixel, Window,
};

pub struct SpellWinAdapter {
    pub window: Window,
    pub rendered: SoftwareRenderer,
    pub size: PhysicalSize, //I am not adding any more properties for now and not puttinting it in a
    // pub currently_displayed_buffer: &'a mut [Rgb8Pixel],
    // pub work_buffer: &'a mut [Rgb8Pixel],
    pub needs_redraw: Cell<bool>,
}

impl SpellWinAdapter {
    pub fn new(
        width: u32,
        height: u32,
        // buffer1: &'static mut [Rgb8Pixel],
        // buffer2: &'static mut [Rgb8Pixel], /* buffer: Vec<Rgb8Pixel>*/
    ) -> Rc<Self> {
        Rc::<SpellWinAdapter>::new_cyclic(|adapter| SpellWinAdapter {
            window: Window::new(adapter.clone()),
            rendered: SoftwareRenderer::new_with_repaint_buffer_type(
                RepaintBufferType::SwappedBuffers,
            ),
            size: PhysicalSize { width, height },
            needs_redraw: Default::default(),
            // currently_displayed_buffer: buffer1,
            // work_buffer: buffer2,
        })
    }

    // pub fn update_buffer(&mut self, mut work_buffer: Vec<Rgb8Pixel>) -> Vec<Rgb8Pixel> {
    //     self.rendered
    //         .render(&mut work_buffer, self.size.width as usize);
    //     return work_buffer;
    // }
    pub fn draw_if_needed(&self, render_callback: impl FnOnce(&SoftwareRenderer)) -> bool {
        if self.needs_redraw.replace(false) {
            render_callback(&self.rendered);
            true
        } else {
            false
        }
    }
}

impl WindowAdapter for SpellWinAdapter {
    fn window(&self) -> &Window {
        &self.window
    }

    fn size(&self) -> PhysicalSize {
        // This value have to be made dynamic by using `xandr`
        PhysicalSize {
            width: self.size.width,
            height: self.size.height,
        }
    }

    fn renderer(&self) -> &dyn slint::platform::Renderer {
        &self.rendered
    }

    fn request_redraw(&self) {
        self.needs_redraw.set(true);
    }
}

pub struct SlintLayerShell {
    pub window_adapter: Rc<SpellWinAdapter>,
}

impl Platform for SlintLayerShell {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window_adapter.clone())
    }

    // THis function doesn't only run the event loop. It i also responsible for
    //the creation of variables and their use in various sector.
    // fn run_event_loop(&self) -> Result<(), slint::PlatformError> {
    // }
}

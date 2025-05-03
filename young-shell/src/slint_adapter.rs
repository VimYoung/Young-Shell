use std::rc::Rc;

use slint::{
    platform::{
        software_renderer::{
            RepaintBufferType::{self},
            SoftwareRenderer,
        },
        Platform, WindowAdapter,
    },
    PhysicalSize, Window,
};

pub struct SpellWinAdapter {
    pub window: Window,
    pub rendered: SoftwareRenderer,
    pub size: PhysicalSize, //I am not adding any more properties for now and not puttinting it in a
}

impl SpellWinAdapter {
    pub fn new(repaint_buffer_type: RepaintBufferType, width: u32, height: u32) -> Rc<Self> {
        Rc::<SpellWinAdapter>::new_cyclic(|adapter| SpellWinAdapter {
            window: Window::new(adapter.clone()),
            rendered: SoftwareRenderer::new_with_repaint_buffer_type(repaint_buffer_type),
            size: PhysicalSize { width, height },
        })
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

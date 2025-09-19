use std::sync::{Mutex, OnceLock};

use winit::event::Touch;

/// InputHandler holds a reference to the current InputHandler
///
/// The graph:
///
/// Input - send events -> InputHandler - call handler -> Output
pub(crate) struct InputHandler {
    /// Last event, usually a start event
    event: Option<(f64, f64)>,
    handler: Option<Box<(dyn Fn(f64, f64) + Send + Sync)>>,
}

static INPUT_HANDLER: OnceLock<Mutex<InputHandler>> = OnceLock::new();

/// InputHandler handles user input events
///
/// This is a singleton, only one instance is in a process
impl InputHandler {
    /// Create a new InputHandler
    ///
    /// Internal use only
    ///
    /// Call get to get the current InputHandler
    fn new() -> Self {
        InputHandler {
            event: None,
            handler: None,
        }
    }

    /// Get the current InputHandler
    ///
    /// If one does not exist, it will be created
    pub fn get() -> &'static Mutex<InputHandler> {
        INPUT_HANDLER.get_or_init(|| Mutex::new(InputHandler::new()))
    }

    /// Add Event to the InputHandler
    ///
    /// This is called by the event loop, when a touch event occurs, add to the queue,
    /// and once the end event occurs, call the handler
    ///
    /// Example:
    /// ```rust
    /// let mut input_handler = InputHandler::get().lock().unwrap();
    /// input_handler.add_event(touch);
    /// ```
    pub fn add_event(&mut self, touch: Touch) {
        let (touch_x, touch_y) = (touch.location.x, touch.location.y);

        match touch.phase {
            winit::event::TouchPhase::Started => {
                self.event = Some((touch_x, touch_y));
            }
            winit::event::TouchPhase::Moved => {}
            winit::event::TouchPhase::Ended => {
                let (start_x, start_y) = self.event.unwrap_or_default();
                self.event = None;
                if let Some(handler) = &self.handler {
                    handler(touch_x - start_x, touch_y - start_y);
                }
            }
            winit::event::TouchPhase::Cancelled => {
                self.event = None;
            }
        }
    }

    /// Register a handler for the InputHandler
    ///
    /// Example:
    /// ```rust
    /// let mut input_handler = InputHandler::get().lock().unwrap();
    /// input_handler.register_handler(move |x, y| {
    ///     println!("Touch delta: {}, {}", x, y);
    /// });
    /// ```
    pub fn register_handler<F>(&mut self, callback: F)
    where
        F: Fn(f64, f64) + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(callback));
    }
}

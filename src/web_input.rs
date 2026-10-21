//! Native touch editing for canvas number fields. Focus starts in the browser's tap handler
//! so mobile browsers can open their software keyboard without an asynchronous focus request.

use three_d::{Event, MouseButton};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/web_input.js")]
extern "C" {
    pub fn begin_frame(width: f32, height: f32);
    pub fn end_frame();
    pub fn is_editing(id: &str) -> bool;
    fn is_touch_pointer() -> bool;
    pub fn take_value(id: &str) -> Option<String>;
    pub fn register_number(id: &str, value: &str, label: &str, bounds: &[f32]);
}

pub fn prepare_events(events: &mut Vec<Event>) {
    if !is_touch_pointer() {
        return;
    }

    // three-d converts touches to mouse events without clearing the pointer on lift.
    // Egui needs PointerGone after release, or the next contact's delta includes the
    // distance from the old finger position and immediately scrolls the background.
    for index in (0..events.len()).rev() {
        if matches!(
            events[index],
            Event::MouseRelease {
                button: MouseButton::Left,
                ..
            }
        ) {
            events.insert(index + 1, Event::MouseLeave);
        }
    }
}

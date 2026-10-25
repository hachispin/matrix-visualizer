use three_d::{Event, MouseButton};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/web_input.js")]
extern "C" {
    pub fn begin_frame(width: f32, height: f32);
    pub fn end_frame();
    pub fn is_editing(id: &str) -> bool;
    pub fn has_touch_pointer() -> bool;
    fn is_touch_pointer() -> bool;
    pub fn take_value(id: &str) -> Option<String>;
    pub fn register_number(
        id: &str,
        value: &str,
        label: &str,
        bounds: &[f32],
        last_in_sequence: Option<bool>,
    );
}

pub fn prepare_events(events: &mut Vec<Event>) {
    if !is_touch_pointer() {
        return;
    }

    // three-d does not clear pointer on lift (touch is converted to mouse events) so
    // it causes some scrolling issues. to fix this, i just insert the event when needed
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

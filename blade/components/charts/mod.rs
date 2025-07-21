pub mod linechart;
pub mod piechart;

pub(super) fn get_mouse_position_from_event(event: &web_sys::MouseEvent) -> (f64, f64) {
    (event.client_x() as f64, event.client_y() as f64)
}

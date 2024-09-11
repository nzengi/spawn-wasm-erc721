use web_sys::console;

/// Yardımcı fonksiyon: Olay günlüğü
pub fn log_event(event: &str, details: &str) {
    console::log_2(&event.into(), &details.into());
}

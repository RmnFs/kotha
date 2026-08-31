//! English tray menu strings generated from the frontend catalog at build time.

use once_cell::sync::Lazy;
use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/tray_translations.rs"));

pub fn get_tray_translations() -> TrayStrings {
    TRANSLATIONS
        .get("en")
        .cloned()
        .expect("English tray translations must exist")
}

#[cfg(test)]
mod tests {
    use super::get_tray_translations;

    #[test]
    fn loads_english_tray_strings() {
        let strings = get_tray_translations();
        assert_eq!(strings.settings, "Settings...");
        assert_eq!(strings.quit, "Quit");
    }
}

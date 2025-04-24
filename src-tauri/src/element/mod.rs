pub mod element;

pub use element::{UIElement, WINDOWS_UI_ELEMENTS_MAP_STORAGE};

use crate::config;
use element::collect_ui_elements;
use log::info;
use std::time::Duration;

pub fn setup_ui_collection(config: &config::Config) {
    let collect_interval = config.ui_automation.collect_interval;
    std::thread::spawn(move || {
        info!("[✓] UI元素收集线程已启动");
        loop {
            std::thread::sleep(Duration::from_millis(collect_interval));
            collect_ui_elements();
        }
    });
}

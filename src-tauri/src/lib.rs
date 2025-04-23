pub mod monitors;
pub mod app_windows;
pub mod elements;
pub mod hints;
pub mod utils;
pub mod inputs;
pub mod configs;

// Re-export commonly used types
pub use monitors::MonitorInfo;
pub use app_windows::WindowElement;
pub use elements::UIElement;
pub use hints::generator::Hint;
pub use utils::Rect;
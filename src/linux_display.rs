//! Display backend workaround for WebKitGTK under Wayland.

/// On Wayland, WebKitGTK can hit protocol errors and GBM buffer failures
/// (seen with NVIDIA drivers). Force the X11 backend through XWayland and disable
/// GPU-accelerated compositing to ensure reliable rendering.
///
/// Call it first thing in `main`, before any thread exists: it sets
/// environment variables.
pub fn configure() {
    if std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|v| v == "wayland")
    {
        std::env::set_var("GDK_BACKEND", "x11");
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }
}

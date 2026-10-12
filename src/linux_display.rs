//! Display workaround for WebKitGTK under Wayland.

/// On Wayland, WebKitGTK's DMA-BUF renderer fails with a protocol error
/// (seen with NVIDIA drivers), so it and GPU-accelerated compositing stay off.
/// The window stays a native Wayland one: under XWayland, Hyprland ignores the
/// move a drag of the status bar asks for.
///
/// Call it first thing in `main`, before any thread exists: it sets
/// environment variables. A value already set in the environment wins.
pub fn configure() {
    if std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE").is_ok_and(|v| v == "wayland")
    {
        for name in [
            "WEBKIT_DISABLE_DMABUF_RENDERER",
            "WEBKIT_DISABLE_COMPOSITING_MODE",
        ] {
            if std::env::var_os(name).is_none() {
                std::env::set_var(name, "1");
            }
        }
    }
}

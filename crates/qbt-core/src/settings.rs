use serde::{Deserialize, Serialize};

/// Client-only settings. Stored locally, never sent to the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ClientSettings {
    /// `system`, `light` or `dark`.
    pub theme: String,
    /// Qt Quick Controls style override; empty means the platform default.
    pub style: String,
    pub confirm_delete: bool,
    pub confirm_shutdown: bool,
    /// Multiplies the server's refresh interval.
    pub polling_multiplier: f64,
    pub notify_on_completion: bool,
    pub tray_enabled: bool,
    pub register_magnet_handler: bool,
    /// `toggle` or `properties`.
    pub double_click_action: String,
    pub show_toolbar: bool,
    pub show_status_bar: bool,
    pub show_sidebar: bool,
    pub speed_in_title: bool,
    /// Visible transfer-list column keys, comma-separated; empty means the defaults.
    pub transfer_columns: String,
    /// Transfer-list column widths as `key:width` pairs, comma-separated.
    pub column_widths: String,
    /// Visual order of the transfer-list columns, as comma-separated keys (empty = default order).
    pub column_order: String,
    pub show_log_tab: bool,
    pub show_search_tab: bool,
    pub show_rss_tab: bool,
    /// Opt-in: share the transfer-list layout between machines through the server's
    /// `clientdata` store (WebAPI 2.13.1+).
    pub sync_ui_state: bool,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            style: String::new(),
            confirm_delete: true,
            confirm_shutdown: true,
            polling_multiplier: 1.0,
            notify_on_completion: true,
            tray_enabled: false,
            register_magnet_handler: false,
            double_click_action: "toggle".into(),
            show_toolbar: true,
            show_status_bar: true,
            show_sidebar: true,
            speed_in_title: false,
            transfer_columns: String::new(),
            column_widths: String::new(),
            column_order: String::new(),
            show_log_tab: true,
            show_search_tab: true,
            show_rss_tab: true,
            sync_ui_state: false,
        }
    }
}

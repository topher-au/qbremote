//! Typed view of `app/preferences` (243 keys, WebAPI 2.17.0) and a typed write patch.
//! Generated from the inventory; types come from the source's `setPreferencesAction` conversions
//! (cross-checked with the 5.0 wiki). Decoding never fails on a type mismatch: the field is `None`.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

fn lenient<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let v = Option::<Value>::deserialize(d)?;
    Ok(v.and_then(|v| serde_json::from_value(v).ok()))
}

/// Every preference key, in source order (see `docs/api-inventory.md`).
pub const KEYS: &[&str] = &[
    "locale",
    "performance_warning",
    "status_bar_external_ip",
    "confirm_torrent_deletion",
    "store_search_jobs",
    "store_search_job_results",
    "file_log_enabled",
    "file_log_path",
    "file_log_backup_enabled",
    "file_log_max_size",
    "file_log_delete_old",
    "file_log_age",
    "file_log_age_type",
    "delete_torrent_content_files",
    "torrent_content_layout",
    "add_to_top_of_queue",
    "add_stopped_enabled",
    "torrent_stop_condition",
    "merge_trackers",
    "auto_delete_mode",
    "preallocate_all",
    "incomplete_files_ext",
    "use_unwanted_folder",
    "auto_tmm_enabled",
    "torrent_changed_tmm_enabled",
    "save_path_changed_tmm_enabled",
    "category_changed_tmm_enabled",
    "save_path",
    "temp_path_enabled",
    "temp_path",
    "use_category_paths_in_manual_mode",
    "torrent_files_backup_enabled",
    "torrent_files_backup_dir",
    "torrent_files_finished_backup_dir_enabled",
    "torrent_files_finished_backup_dir",
    "remove_torrent_file_backup",
    "scan_dirs",
    "excluded_file_names_enabled",
    "excluded_file_names",
    "exclude_files_without_extension",
    "mail_notification_enabled",
    "mail_notification_sender",
    "mail_notification_email",
    "mail_notification_smtp",
    "mail_notification_encryption_type",
    "mail_notification_auth_enabled",
    "mail_notification_username",
    "mail_notification_password",
    "autorun_on_torrent_added_enabled",
    "autorun_on_torrent_added_program",
    "autorun_enabled",
    "autorun_program",
    "listen_port",
    "ssl_enabled",
    "ssl_listen_port",
    "random_port",
    "upnp",
    "max_connec",
    "max_connec_per_torrent",
    "max_uploads",
    "max_uploads_per_torrent",
    "i2p_enabled",
    "i2p_address",
    "i2p_port",
    "i2p_mixed_mode",
    "i2p_pex_enabled",
    "i2p_inbound_quantity",
    "i2p_outbound_quantity",
    "i2p_inbound_length",
    "i2p_outbound_length",
    "i2p_inbound_length_variance",
    "i2p_outbound_length_variance",
    "proxy_type",
    "proxy_ip",
    "proxy_port",
    "proxy_auth_enabled",
    "proxy_username",
    "proxy_password",
    "proxy_hostname_lookup",
    "proxy_bittorrent",
    "proxy_peer_connections",
    "proxy_rss",
    "proxy_misc",
    "ip_filter_enabled",
    "ip_filter_path",
    "ip_filter_trackers",
    "banned_IPs",
    "dl_limit",
    "up_limit",
    "alt_dl_limit",
    "alt_up_limit",
    "bittorrent_protocol",
    "limit_utp_rate",
    "limit_tcp_overhead",
    "limit_lan_peers",
    "scheduler_enabled",
    "schedule_from_hour",
    "schedule_from_min",
    "schedule_to_hour",
    "schedule_to_min",
    "scheduler_days",
    "dht",
    "pex",
    "lsd",
    "encryption",
    "anonymous_mode",
    "max_active_checking_torrents",
    "queueing_enabled",
    "max_active_downloads",
    "max_active_torrents",
    "max_active_uploads",
    "dont_count_slow_torrents",
    "slow_torrent_dl_rate_threshold",
    "slow_torrent_ul_rate_threshold",
    "slow_torrent_inactive_timer",
    "max_ratio_enabled",
    "max_ratio",
    "max_seeding_time_enabled",
    "max_seeding_time",
    "max_inactive_seeding_time_enabled",
    "max_inactive_seeding_time",
    "share_limits_mode",
    "max_ratio_act",
    "add_trackers_enabled",
    "add_trackers",
    "add_trackers_from_url_enabled",
    "add_trackers_url",
    "add_trackers_url_list",
    "web_ui_domain_list",
    "web_ui_address",
    "web_ui_port",
    "web_ui_upnp",
    "use_https",
    "web_ui_https_cert_path",
    "web_ui_https_key_path",
    "web_ui_username",
    "bypass_local_auth",
    "bypass_auth_subnet_whitelist_enabled",
    "bypass_auth_subnet_whitelist",
    "web_ui_max_auth_fail_count",
    "web_ui_ban_duration",
    "web_ui_session_timeout",
    "web_ui_sessions_count_limit",
    "web_ui_api_key",
    "alternative_webui_enabled",
    "alternative_webui_path",
    "web_ui_clickjacking_protection_enabled",
    "web_ui_csrf_protection_enabled",
    "web_ui_secure_cookie_enabled",
    "web_ui_host_header_validation_enabled",
    "web_ui_use_custom_http_headers_enabled",
    "web_ui_custom_http_headers",
    "web_ui_reverse_proxy_enabled",
    "web_ui_reverse_proxies_list",
    "dyndns_enabled",
    "dyndns_service",
    "dyndns_username",
    "dyndns_password",
    "dyndns_domain",
    "rss_refresh_interval",
    "rss_fetch_delay",
    "rss_max_articles_per_feed",
    "rss_processing_enabled",
    "rss_auto_downloading_enabled",
    "rss_download_repack_proper_episodes",
    "rss_smart_episode_filters",
    "resume_data_storage_type",
    "torrent_content_remove_option",
    "memory_working_set_limit",
    "current_network_interface",
    "current_interface_name",
    "current_interface_address",
    "save_resume_data_interval",
    "save_statistics_interval",
    "torrent_file_size_limit",
    "confirm_torrent_recheck",
    "recheck_completed_torrents",
    "app_instance_name",
    "refresh_interval",
    "resolve_peer_host_names",
    "resolve_peer_countries",
    "confirm_remove_all_tags",
    "confirm_remove_tracker_from_all_torrents",
    "reannounce_when_address_changed",
    "enable_embedded_tracker",
    "embedded_tracker_port",
    "embedded_tracker_port_forwarding",
    "mark_of_the_web",
    "ignore_ssl_errors",
    "python_executable_path",
    "start_paused",
    "shutdown_timeout",
    "bdecode_depth_limit",
    "bdecode_token_limit",
    "load_torrent_max_directory_depth",
    "async_io_threads",
    "hashing_threads",
    "file_pool_size",
    "checking_memory_use",
    "disk_cache",
    "disk_cache_ttl",
    "disk_queue_size",
    "disk_io_type",
    "disk_io_read_mode",
    "disk_io_write_mode",
    "enable_coalesce_read_write",
    "enable_piece_extent_affinity",
    "enable_upload_suggestions",
    "send_buffer_watermark",
    "send_buffer_low_watermark",
    "send_buffer_watermark_factor",
    "connection_speed",
    "seeding_outgoing_connections",
    "socket_send_buffer_size",
    "socket_receive_buffer_size",
    "socket_backlog_size",
    "outgoing_ports_min",
    "outgoing_ports_max",
    "upnp_lease_duration",
    "peer_tos",
    "utp_tcp_mixed_mode",
    "hostname_cache_ttl",
    "idn_support_enabled",
    "enable_multi_connections_from_same_ip",
    "enable_multi_connections_from_same_peer_id",
    "validate_https_tracker_certificate",
    "ssrf_mitigation",
    "block_peers_on_privileged_ports",
    "upload_slots_behavior",
    "upload_choking_algorithm",
    "announce_to_all_trackers",
    "announce_to_all_tiers",
    "announce_ip",
    "announce_port",
    "max_concurrent_http_announces",
    "stop_tracker_timeout",
    "peer_turnover",
    "peer_turnover_cutoff",
    "peer_turnover_interval",
    "request_queue_size",
    "max_outstanding_block_requests",
    "dht_bootstrap_nodes",
    "webtorrent_stun_server",
];

/// Keys that hold secrets; redacted in `Debug` and never logged.
pub const SECRET_KEYS: &[&str] = &[
    "web_ui_api_key",
    "web_ui_password",
    "proxy_password",
    "dyndns_password",
    "mail_notification_password",
];

#[derive(Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct TypedPreferences {
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub locale: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub performance_warning: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub status_bar_external_ip: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirm_torrent_deletion: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub store_search_jobs: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub store_search_job_results: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_backup_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_max_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_delete_old: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_age: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_log_age_type: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub delete_torrent_content_files: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_content_layout: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_to_top_of_queue: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_stopped_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_stop_condition: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub merge_trackers: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_delete_mode: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub preallocate_all: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub incomplete_files_ext: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub use_unwanted_folder: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_tmm_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_changed_tmm_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_path_changed_tmm_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub category_changed_tmm_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub temp_path_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub temp_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub use_category_paths_in_manual_mode: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_files_backup_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_files_backup_dir: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_files_finished_backup_dir_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_files_finished_backup_dir: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub remove_torrent_file_backup: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub scan_dirs: Option<Value>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub excluded_file_names_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub excluded_file_names: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub exclude_files_without_extension: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_sender: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_email: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_smtp: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_encryption_type: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_auth_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_username: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mail_notification_password: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub autorun_on_torrent_added_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub autorun_on_torrent_added_program: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub autorun_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub autorun_program: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub listen_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ssl_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ssl_listen_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub random_port: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub upnp: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_connec: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_connec_per_torrent: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_uploads: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_uploads_per_torrent: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_address: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_mixed_mode: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_pex_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_inbound_quantity: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_outbound_quantity: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_inbound_length: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_outbound_length: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_inbound_length_variance: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub i2p_outbound_length_variance: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_type: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_ip: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_auth_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_username: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_password: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_hostname_lookup: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_bittorrent: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_peer_connections: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_rss: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub proxy_misc: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ip_filter_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ip_filter_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ip_filter_trackers: Option<bool>,
    #[serde(rename = "banned_IPs")]
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub banned_ips: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dl_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub up_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub alt_dl_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub alt_up_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub bittorrent_protocol: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_utp_rate: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_tcp_overhead: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_lan_peers: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduler_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub schedule_from_hour: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub schedule_from_min: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub schedule_to_hour: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub schedule_to_min: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduler_days: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dht: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub pex: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub lsd: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub encryption: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub anonymous_mode: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_active_checking_torrents: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub queueing_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_active_downloads: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_active_torrents: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_active_uploads: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dont_count_slow_torrents: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub slow_torrent_dl_rate_threshold: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub slow_torrent_ul_rate_threshold: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub slow_torrent_inactive_timer: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_ratio_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_ratio: Option<f64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_seeding_time_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_seeding_time: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_inactive_seeding_time_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_inactive_seeding_time: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub share_limits_mode: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_ratio_act: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_trackers_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_trackers: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_trackers_from_url_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_trackers_url: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub add_trackers_url_list: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_domain_list: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_address: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_upnp: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub use_https: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_https_cert_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_https_key_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_username: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub bypass_local_auth: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub bypass_auth_subnet_whitelist_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub bypass_auth_subnet_whitelist: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_max_auth_fail_count: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_ban_duration: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_session_timeout: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_sessions_count_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_api_key: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub alternative_webui_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub alternative_webui_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_clickjacking_protection_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_csrf_protection_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_secure_cookie_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_host_header_validation_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_use_custom_http_headers_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_custom_http_headers: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_reverse_proxy_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub web_ui_reverse_proxies_list: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dyndns_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dyndns_service: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dyndns_username: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dyndns_password: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dyndns_domain: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_refresh_interval: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_fetch_delay: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_max_articles_per_feed: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_processing_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_auto_downloading_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_download_repack_proper_episodes: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub rss_smart_episode_filters: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub resume_data_storage_type: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_content_remove_option: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub memory_working_set_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub current_network_interface: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub current_interface_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub current_interface_address: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_resume_data_interval: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub save_statistics_interval: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub torrent_file_size_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirm_torrent_recheck: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub recheck_completed_torrents: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub app_instance_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub refresh_interval: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub resolve_peer_host_names: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub resolve_peer_countries: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirm_remove_all_tags: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirm_remove_tracker_from_all_torrents: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub reannounce_when_address_changed: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_embedded_tracker: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub embedded_tracker_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub embedded_tracker_port_forwarding: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub mark_of_the_web: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ignore_ssl_errors: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub python_executable_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_paused: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub shutdown_timeout: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub bdecode_depth_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub bdecode_token_limit: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub load_torrent_max_directory_depth: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub async_io_threads: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub hashing_threads: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub file_pool_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub checking_memory_use: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub disk_cache: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub disk_cache_ttl: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub disk_queue_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub disk_io_type: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub disk_io_read_mode: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub disk_io_write_mode: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_coalesce_read_write: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_piece_extent_affinity: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_upload_suggestions: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub send_buffer_watermark: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub send_buffer_low_watermark: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub send_buffer_watermark_factor: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub connection_speed: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub seeding_outgoing_connections: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub socket_send_buffer_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub socket_receive_buffer_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub socket_backlog_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub outgoing_ports_min: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub outgoing_ports_max: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub upnp_lease_duration: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub peer_tos: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub utp_tcp_mixed_mode: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub hostname_cache_ttl: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub idn_support_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_multi_connections_from_same_ip: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub enable_multi_connections_from_same_peer_id: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub validate_https_tracker_certificate: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub ssrf_mitigation: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub block_peers_on_privileged_ports: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub upload_slots_behavior: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub upload_choking_algorithm: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub announce_to_all_trackers: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub announce_to_all_tiers: Option<bool>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub announce_ip: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub announce_port: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_concurrent_http_announces: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub stop_tracker_timeout: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub peer_turnover: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub peer_turnover_cutoff: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub peer_turnover_interval: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub request_queue_size: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_outstanding_block_requests: Option<i64>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub dht_bootstrap_nodes: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub webtorrent_stun_server: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl fmt::Debug for TypedPreferences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = serde_json::to_value(self).unwrap_or(Value::Null);
        let mut m = f.debug_map();
        if let Value::Object(o) = v {
            for (k, x) in o {
                if crate::models::app::is_secret_key(&k) {
                    m.entry(&k, &"<redacted>");
                } else {
                    m.entry(&k, &x);
                }
            }
        }
        m.finish()
    }
}

/// Builder for `app/setPreferences`: only the keys you set are sent.
#[derive(Clone, Default)]
pub struct PreferencesPatch(Map<String, Value>);

impl PreferencesPatch {
    pub fn new() -> Self {
        Self::default()
    }
    /// The JSON object for the `json` form field.
    pub fn into_map(self) -> Map<String, Value> {
        self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Set any key by name (for keys a newer server adds).
    pub fn set_raw(mut self, key: &str, value: Value) -> Self {
        self.0.insert(key.to_owned(), value);
        self
    }
    pub fn locale(mut self, v: impl Into<String>) -> Self {
        self.0.insert("locale".into(), Value::String(v.into()));
        self
    }
    pub fn performance_warning(mut self, v: bool) -> Self {
        self.0.insert("performance_warning".into(), Value::from(v));
        self
    }
    pub fn status_bar_external_ip(mut self, v: bool) -> Self {
        self.0
            .insert("status_bar_external_ip".into(), Value::from(v));
        self
    }
    pub fn confirm_torrent_deletion(mut self, v: bool) -> Self {
        self.0
            .insert("confirm_torrent_deletion".into(), Value::from(v));
        self
    }
    pub fn store_search_jobs(mut self, v: bool) -> Self {
        self.0.insert("store_search_jobs".into(), Value::from(v));
        self
    }
    pub fn store_search_job_results(mut self, v: bool) -> Self {
        self.0
            .insert("store_search_job_results".into(), Value::from(v));
        self
    }
    pub fn file_log_enabled(mut self, v: bool) -> Self {
        self.0.insert("file_log_enabled".into(), Value::from(v));
        self
    }
    pub fn file_log_path(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("file_log_path".into(), Value::String(v.into()));
        self
    }
    pub fn file_log_backup_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("file_log_backup_enabled".into(), Value::from(v));
        self
    }
    pub fn file_log_max_size(mut self, v: i64) -> Self {
        self.0.insert("file_log_max_size".into(), Value::from(v));
        self
    }
    pub fn file_log_delete_old(mut self, v: bool) -> Self {
        self.0.insert("file_log_delete_old".into(), Value::from(v));
        self
    }
    pub fn file_log_age(mut self, v: i64) -> Self {
        self.0.insert("file_log_age".into(), Value::from(v));
        self
    }
    pub fn file_log_age_type(mut self, v: i64) -> Self {
        self.0.insert("file_log_age_type".into(), Value::from(v));
        self
    }
    pub fn delete_torrent_content_files(mut self, v: bool) -> Self {
        self.0
            .insert("delete_torrent_content_files".into(), Value::from(v));
        self
    }
    pub fn torrent_content_layout(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("torrent_content_layout".into(), Value::String(v.into()));
        self
    }
    pub fn add_to_top_of_queue(mut self, v: bool) -> Self {
        self.0.insert("add_to_top_of_queue".into(), Value::from(v));
        self
    }
    pub fn add_stopped_enabled(mut self, v: bool) -> Self {
        self.0.insert("add_stopped_enabled".into(), Value::from(v));
        self
    }
    pub fn torrent_stop_condition(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("torrent_stop_condition".into(), Value::String(v.into()));
        self
    }
    pub fn merge_trackers(mut self, v: bool) -> Self {
        self.0.insert("merge_trackers".into(), Value::from(v));
        self
    }
    pub fn auto_delete_mode(mut self, v: i64) -> Self {
        self.0.insert("auto_delete_mode".into(), Value::from(v));
        self
    }
    pub fn preallocate_all(mut self, v: bool) -> Self {
        self.0.insert("preallocate_all".into(), Value::from(v));
        self
    }
    pub fn incomplete_files_ext(mut self, v: bool) -> Self {
        self.0.insert("incomplete_files_ext".into(), Value::from(v));
        self
    }
    pub fn use_unwanted_folder(mut self, v: bool) -> Self {
        self.0.insert("use_unwanted_folder".into(), Value::from(v));
        self
    }
    pub fn auto_tmm_enabled(mut self, v: bool) -> Self {
        self.0.insert("auto_tmm_enabled".into(), Value::from(v));
        self
    }
    pub fn torrent_changed_tmm_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("torrent_changed_tmm_enabled".into(), Value::from(v));
        self
    }
    pub fn save_path_changed_tmm_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("save_path_changed_tmm_enabled".into(), Value::from(v));
        self
    }
    pub fn category_changed_tmm_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("category_changed_tmm_enabled".into(), Value::from(v));
        self
    }
    pub fn save_path(mut self, v: impl Into<String>) -> Self {
        self.0.insert("save_path".into(), Value::String(v.into()));
        self
    }
    pub fn temp_path_enabled(mut self, v: bool) -> Self {
        self.0.insert("temp_path_enabled".into(), Value::from(v));
        self
    }
    pub fn temp_path(mut self, v: impl Into<String>) -> Self {
        self.0.insert("temp_path".into(), Value::String(v.into()));
        self
    }
    pub fn use_category_paths_in_manual_mode(mut self, v: bool) -> Self {
        self.0
            .insert("use_category_paths_in_manual_mode".into(), Value::from(v));
        self
    }
    pub fn torrent_files_backup_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("torrent_files_backup_enabled".into(), Value::from(v));
        self
    }
    pub fn torrent_files_backup_dir(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("torrent_files_backup_dir".into(), Value::String(v.into()));
        self
    }
    pub fn torrent_files_finished_backup_dir_enabled(mut self, v: bool) -> Self {
        self.0.insert(
            "torrent_files_finished_backup_dir_enabled".into(),
            Value::from(v),
        );
        self
    }
    pub fn torrent_files_finished_backup_dir(mut self, v: impl Into<String>) -> Self {
        self.0.insert(
            "torrent_files_finished_backup_dir".into(),
            Value::String(v.into()),
        );
        self
    }
    pub fn remove_torrent_file_backup(mut self, v: bool) -> Self {
        self.0
            .insert("remove_torrent_file_backup".into(), Value::from(v));
        self
    }
    pub fn scan_dirs(mut self, v: Value) -> Self {
        self.0.insert("scan_dirs".into(), v);
        self
    }
    pub fn excluded_file_names_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("excluded_file_names_enabled".into(), Value::from(v));
        self
    }
    pub fn excluded_file_names(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("excluded_file_names".into(), Value::String(v.into()));
        self
    }
    pub fn exclude_files_without_extension(mut self, v: bool) -> Self {
        self.0
            .insert("exclude_files_without_extension".into(), Value::from(v));
        self
    }
    pub fn mail_notification_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("mail_notification_enabled".into(), Value::from(v));
        self
    }
    pub fn mail_notification_sender(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("mail_notification_sender".into(), Value::String(v.into()));
        self
    }
    pub fn mail_notification_email(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("mail_notification_email".into(), Value::String(v.into()));
        self
    }
    pub fn mail_notification_smtp(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("mail_notification_smtp".into(), Value::String(v.into()));
        self
    }
    pub fn mail_notification_encryption_type(mut self, v: impl Into<String>) -> Self {
        self.0.insert(
            "mail_notification_encryption_type".into(),
            Value::String(v.into()),
        );
        self
    }
    pub fn mail_notification_auth_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("mail_notification_auth_enabled".into(), Value::from(v));
        self
    }
    pub fn mail_notification_username(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("mail_notification_username".into(), Value::String(v.into()));
        self
    }
    pub fn mail_notification_password(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("mail_notification_password".into(), Value::String(v.into()));
        self
    }
    pub fn autorun_on_torrent_added_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("autorun_on_torrent_added_enabled".into(), Value::from(v));
        self
    }
    pub fn autorun_on_torrent_added_program(mut self, v: impl Into<String>) -> Self {
        self.0.insert(
            "autorun_on_torrent_added_program".into(),
            Value::String(v.into()),
        );
        self
    }
    pub fn autorun_enabled(mut self, v: bool) -> Self {
        self.0.insert("autorun_enabled".into(), Value::from(v));
        self
    }
    pub fn autorun_program(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("autorun_program".into(), Value::String(v.into()));
        self
    }
    pub fn listen_port(mut self, v: i64) -> Self {
        self.0.insert("listen_port".into(), Value::from(v));
        self
    }
    pub fn ssl_enabled(mut self, v: bool) -> Self {
        self.0.insert("ssl_enabled".into(), Value::from(v));
        self
    }
    pub fn ssl_listen_port(mut self, v: i64) -> Self {
        self.0.insert("ssl_listen_port".into(), Value::from(v));
        self
    }
    pub fn random_port(mut self, v: bool) -> Self {
        self.0.insert("random_port".into(), Value::from(v));
        self
    }
    pub fn upnp(mut self, v: bool) -> Self {
        self.0.insert("upnp".into(), Value::from(v));
        self
    }
    pub fn max_connec(mut self, v: i64) -> Self {
        self.0.insert("max_connec".into(), Value::from(v));
        self
    }
    pub fn max_connec_per_torrent(mut self, v: i64) -> Self {
        self.0
            .insert("max_connec_per_torrent".into(), Value::from(v));
        self
    }
    pub fn max_uploads(mut self, v: i64) -> Self {
        self.0.insert("max_uploads".into(), Value::from(v));
        self
    }
    pub fn max_uploads_per_torrent(mut self, v: i64) -> Self {
        self.0
            .insert("max_uploads_per_torrent".into(), Value::from(v));
        self
    }
    pub fn i2p_enabled(mut self, v: bool) -> Self {
        self.0.insert("i2p_enabled".into(), Value::from(v));
        self
    }
    pub fn i2p_address(mut self, v: impl Into<String>) -> Self {
        self.0.insert("i2p_address".into(), Value::String(v.into()));
        self
    }
    pub fn i2p_port(mut self, v: i64) -> Self {
        self.0.insert("i2p_port".into(), Value::from(v));
        self
    }
    pub fn i2p_mixed_mode(mut self, v: bool) -> Self {
        self.0.insert("i2p_mixed_mode".into(), Value::from(v));
        self
    }
    pub fn i2p_pex_enabled(mut self, v: bool) -> Self {
        self.0.insert("i2p_pex_enabled".into(), Value::from(v));
        self
    }
    pub fn i2p_inbound_quantity(mut self, v: i64) -> Self {
        self.0.insert("i2p_inbound_quantity".into(), Value::from(v));
        self
    }
    pub fn i2p_outbound_quantity(mut self, v: i64) -> Self {
        self.0
            .insert("i2p_outbound_quantity".into(), Value::from(v));
        self
    }
    pub fn i2p_inbound_length(mut self, v: i64) -> Self {
        self.0.insert("i2p_inbound_length".into(), Value::from(v));
        self
    }
    pub fn i2p_outbound_length(mut self, v: i64) -> Self {
        self.0.insert("i2p_outbound_length".into(), Value::from(v));
        self
    }
    pub fn i2p_inbound_length_variance(mut self, v: i64) -> Self {
        self.0
            .insert("i2p_inbound_length_variance".into(), Value::from(v));
        self
    }
    pub fn i2p_outbound_length_variance(mut self, v: i64) -> Self {
        self.0
            .insert("i2p_outbound_length_variance".into(), Value::from(v));
        self
    }
    pub fn proxy_type(mut self, v: impl Into<String>) -> Self {
        self.0.insert("proxy_type".into(), Value::String(v.into()));
        self
    }
    pub fn proxy_ip(mut self, v: impl Into<String>) -> Self {
        self.0.insert("proxy_ip".into(), Value::String(v.into()));
        self
    }
    pub fn proxy_port(mut self, v: i64) -> Self {
        self.0.insert("proxy_port".into(), Value::from(v));
        self
    }
    pub fn proxy_auth_enabled(mut self, v: bool) -> Self {
        self.0.insert("proxy_auth_enabled".into(), Value::from(v));
        self
    }
    pub fn proxy_username(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("proxy_username".into(), Value::String(v.into()));
        self
    }
    pub fn proxy_password(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("proxy_password".into(), Value::String(v.into()));
        self
    }
    pub fn proxy_hostname_lookup(mut self, v: bool) -> Self {
        self.0
            .insert("proxy_hostname_lookup".into(), Value::from(v));
        self
    }
    pub fn proxy_bittorrent(mut self, v: bool) -> Self {
        self.0.insert("proxy_bittorrent".into(), Value::from(v));
        self
    }
    pub fn proxy_peer_connections(mut self, v: bool) -> Self {
        self.0
            .insert("proxy_peer_connections".into(), Value::from(v));
        self
    }
    pub fn proxy_rss(mut self, v: bool) -> Self {
        self.0.insert("proxy_rss".into(), Value::from(v));
        self
    }
    pub fn proxy_misc(mut self, v: bool) -> Self {
        self.0.insert("proxy_misc".into(), Value::from(v));
        self
    }
    pub fn ip_filter_enabled(mut self, v: bool) -> Self {
        self.0.insert("ip_filter_enabled".into(), Value::from(v));
        self
    }
    pub fn ip_filter_path(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("ip_filter_path".into(), Value::String(v.into()));
        self
    }
    pub fn ip_filter_trackers(mut self, v: bool) -> Self {
        self.0.insert("ip_filter_trackers".into(), Value::from(v));
        self
    }
    pub fn banned_ips(mut self, v: impl Into<String>) -> Self {
        self.0.insert("banned_IPs".into(), Value::String(v.into()));
        self
    }
    pub fn dl_limit(mut self, v: i64) -> Self {
        self.0.insert("dl_limit".into(), Value::from(v));
        self
    }
    pub fn up_limit(mut self, v: i64) -> Self {
        self.0.insert("up_limit".into(), Value::from(v));
        self
    }
    pub fn alt_dl_limit(mut self, v: i64) -> Self {
        self.0.insert("alt_dl_limit".into(), Value::from(v));
        self
    }
    pub fn alt_up_limit(mut self, v: i64) -> Self {
        self.0.insert("alt_up_limit".into(), Value::from(v));
        self
    }
    pub fn bittorrent_protocol(mut self, v: i64) -> Self {
        self.0.insert("bittorrent_protocol".into(), Value::from(v));
        self
    }
    pub fn limit_utp_rate(mut self, v: bool) -> Self {
        self.0.insert("limit_utp_rate".into(), Value::from(v));
        self
    }
    pub fn limit_tcp_overhead(mut self, v: bool) -> Self {
        self.0.insert("limit_tcp_overhead".into(), Value::from(v));
        self
    }
    pub fn limit_lan_peers(mut self, v: bool) -> Self {
        self.0.insert("limit_lan_peers".into(), Value::from(v));
        self
    }
    pub fn scheduler_enabled(mut self, v: bool) -> Self {
        self.0.insert("scheduler_enabled".into(), Value::from(v));
        self
    }
    pub fn schedule_from_hour(mut self, v: i64) -> Self {
        self.0.insert("schedule_from_hour".into(), Value::from(v));
        self
    }
    pub fn schedule_from_min(mut self, v: i64) -> Self {
        self.0.insert("schedule_from_min".into(), Value::from(v));
        self
    }
    pub fn schedule_to_hour(mut self, v: i64) -> Self {
        self.0.insert("schedule_to_hour".into(), Value::from(v));
        self
    }
    pub fn schedule_to_min(mut self, v: i64) -> Self {
        self.0.insert("schedule_to_min".into(), Value::from(v));
        self
    }
    pub fn scheduler_days(mut self, v: i64) -> Self {
        self.0.insert("scheduler_days".into(), Value::from(v));
        self
    }
    pub fn dht(mut self, v: bool) -> Self {
        self.0.insert("dht".into(), Value::from(v));
        self
    }
    pub fn pex(mut self, v: bool) -> Self {
        self.0.insert("pex".into(), Value::from(v));
        self
    }
    pub fn lsd(mut self, v: bool) -> Self {
        self.0.insert("lsd".into(), Value::from(v));
        self
    }
    pub fn encryption(mut self, v: i64) -> Self {
        self.0.insert("encryption".into(), Value::from(v));
        self
    }
    pub fn anonymous_mode(mut self, v: bool) -> Self {
        self.0.insert("anonymous_mode".into(), Value::from(v));
        self
    }
    pub fn max_active_checking_torrents(mut self, v: i64) -> Self {
        self.0
            .insert("max_active_checking_torrents".into(), Value::from(v));
        self
    }
    pub fn queueing_enabled(mut self, v: bool) -> Self {
        self.0.insert("queueing_enabled".into(), Value::from(v));
        self
    }
    pub fn max_active_downloads(mut self, v: i64) -> Self {
        self.0.insert("max_active_downloads".into(), Value::from(v));
        self
    }
    pub fn max_active_torrents(mut self, v: i64) -> Self {
        self.0.insert("max_active_torrents".into(), Value::from(v));
        self
    }
    pub fn max_active_uploads(mut self, v: i64) -> Self {
        self.0.insert("max_active_uploads".into(), Value::from(v));
        self
    }
    pub fn dont_count_slow_torrents(mut self, v: bool) -> Self {
        self.0
            .insert("dont_count_slow_torrents".into(), Value::from(v));
        self
    }
    pub fn slow_torrent_dl_rate_threshold(mut self, v: i64) -> Self {
        self.0
            .insert("slow_torrent_dl_rate_threshold".into(), Value::from(v));
        self
    }
    pub fn slow_torrent_ul_rate_threshold(mut self, v: i64) -> Self {
        self.0
            .insert("slow_torrent_ul_rate_threshold".into(), Value::from(v));
        self
    }
    pub fn slow_torrent_inactive_timer(mut self, v: i64) -> Self {
        self.0
            .insert("slow_torrent_inactive_timer".into(), Value::from(v));
        self
    }
    pub fn max_ratio_enabled(mut self, v: bool) -> Self {
        self.0.insert("max_ratio_enabled".into(), Value::from(v));
        self
    }
    pub fn max_ratio(mut self, v: f64) -> Self {
        self.0.insert("max_ratio".into(), Value::from(v));
        self
    }
    pub fn max_seeding_time_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("max_seeding_time_enabled".into(), Value::from(v));
        self
    }
    pub fn max_seeding_time(mut self, v: i64) -> Self {
        self.0.insert("max_seeding_time".into(), Value::from(v));
        self
    }
    pub fn max_inactive_seeding_time_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("max_inactive_seeding_time_enabled".into(), Value::from(v));
        self
    }
    pub fn max_inactive_seeding_time(mut self, v: i64) -> Self {
        self.0
            .insert("max_inactive_seeding_time".into(), Value::from(v));
        self
    }
    pub fn share_limits_mode(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("share_limits_mode".into(), Value::String(v.into()));
        self
    }
    pub fn max_ratio_act(mut self, v: i64) -> Self {
        self.0.insert("max_ratio_act".into(), Value::from(v));
        self
    }
    pub fn add_trackers_enabled(mut self, v: bool) -> Self {
        self.0.insert("add_trackers_enabled".into(), Value::from(v));
        self
    }
    pub fn add_trackers(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("add_trackers".into(), Value::String(v.into()));
        self
    }
    pub fn add_trackers_from_url_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("add_trackers_from_url_enabled".into(), Value::from(v));
        self
    }
    pub fn add_trackers_url(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("add_trackers_url".into(), Value::String(v.into()));
        self
    }
    pub fn add_trackers_url_list(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("add_trackers_url_list".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_domain_list(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_domain_list".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_address(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_address".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_port(mut self, v: i64) -> Self {
        self.0.insert("web_ui_port".into(), Value::from(v));
        self
    }
    pub fn web_ui_upnp(mut self, v: bool) -> Self {
        self.0.insert("web_ui_upnp".into(), Value::from(v));
        self
    }
    pub fn use_https(mut self, v: bool) -> Self {
        self.0.insert("use_https".into(), Value::from(v));
        self
    }
    pub fn web_ui_https_cert_path(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_https_cert_path".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_https_key_path(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_https_key_path".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_username(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_username".into(), Value::String(v.into()));
        self
    }
    pub fn bypass_local_auth(mut self, v: bool) -> Self {
        self.0.insert("bypass_local_auth".into(), Value::from(v));
        self
    }
    pub fn bypass_auth_subnet_whitelist_enabled(mut self, v: bool) -> Self {
        self.0.insert(
            "bypass_auth_subnet_whitelist_enabled".into(),
            Value::from(v),
        );
        self
    }
    pub fn bypass_auth_subnet_whitelist(mut self, v: impl Into<String>) -> Self {
        self.0.insert(
            "bypass_auth_subnet_whitelist".into(),
            Value::String(v.into()),
        );
        self
    }
    pub fn web_ui_max_auth_fail_count(mut self, v: i64) -> Self {
        self.0
            .insert("web_ui_max_auth_fail_count".into(), Value::from(v));
        self
    }
    pub fn web_ui_ban_duration(mut self, v: i64) -> Self {
        self.0.insert("web_ui_ban_duration".into(), Value::from(v));
        self
    }
    pub fn web_ui_session_timeout(mut self, v: i64) -> Self {
        self.0
            .insert("web_ui_session_timeout".into(), Value::from(v));
        self
    }
    pub fn web_ui_sessions_count_limit(mut self, v: i64) -> Self {
        self.0
            .insert("web_ui_sessions_count_limit".into(), Value::from(v));
        self
    }
    pub fn web_ui_api_key(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_api_key".into(), Value::String(v.into()));
        self
    }
    pub fn alternative_webui_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("alternative_webui_enabled".into(), Value::from(v));
        self
    }
    pub fn alternative_webui_path(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("alternative_webui_path".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_clickjacking_protection_enabled(mut self, v: bool) -> Self {
        self.0.insert(
            "web_ui_clickjacking_protection_enabled".into(),
            Value::from(v),
        );
        self
    }
    pub fn web_ui_csrf_protection_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("web_ui_csrf_protection_enabled".into(), Value::from(v));
        self
    }
    pub fn web_ui_secure_cookie_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("web_ui_secure_cookie_enabled".into(), Value::from(v));
        self
    }
    pub fn web_ui_host_header_validation_enabled(mut self, v: bool) -> Self {
        self.0.insert(
            "web_ui_host_header_validation_enabled".into(),
            Value::from(v),
        );
        self
    }
    pub fn web_ui_use_custom_http_headers_enabled(mut self, v: bool) -> Self {
        self.0.insert(
            "web_ui_use_custom_http_headers_enabled".into(),
            Value::from(v),
        );
        self
    }
    pub fn web_ui_custom_http_headers(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_custom_http_headers".into(), Value::String(v.into()));
        self
    }
    pub fn web_ui_reverse_proxy_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("web_ui_reverse_proxy_enabled".into(), Value::from(v));
        self
    }
    pub fn web_ui_reverse_proxies_list(mut self, v: impl Into<String>) -> Self {
        self.0.insert(
            "web_ui_reverse_proxies_list".into(),
            Value::String(v.into()),
        );
        self
    }
    pub fn dyndns_enabled(mut self, v: bool) -> Self {
        self.0.insert("dyndns_enabled".into(), Value::from(v));
        self
    }
    pub fn dyndns_service(mut self, v: i64) -> Self {
        self.0.insert("dyndns_service".into(), Value::from(v));
        self
    }
    pub fn dyndns_username(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("dyndns_username".into(), Value::String(v.into()));
        self
    }
    pub fn dyndns_password(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("dyndns_password".into(), Value::String(v.into()));
        self
    }
    pub fn dyndns_domain(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("dyndns_domain".into(), Value::String(v.into()));
        self
    }
    pub fn rss_refresh_interval(mut self, v: i64) -> Self {
        self.0.insert("rss_refresh_interval".into(), Value::from(v));
        self
    }
    pub fn rss_fetch_delay(mut self, v: i64) -> Self {
        self.0.insert("rss_fetch_delay".into(), Value::from(v));
        self
    }
    pub fn rss_max_articles_per_feed(mut self, v: i64) -> Self {
        self.0
            .insert("rss_max_articles_per_feed".into(), Value::from(v));
        self
    }
    pub fn rss_processing_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("rss_processing_enabled".into(), Value::from(v));
        self
    }
    pub fn rss_auto_downloading_enabled(mut self, v: bool) -> Self {
        self.0
            .insert("rss_auto_downloading_enabled".into(), Value::from(v));
        self
    }
    pub fn rss_download_repack_proper_episodes(mut self, v: bool) -> Self {
        self.0
            .insert("rss_download_repack_proper_episodes".into(), Value::from(v));
        self
    }
    pub fn rss_smart_episode_filters(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("rss_smart_episode_filters".into(), Value::String(v.into()));
        self
    }
    pub fn resume_data_storage_type(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("resume_data_storage_type".into(), Value::String(v.into()));
        self
    }
    pub fn torrent_content_remove_option(mut self, v: impl Into<String>) -> Self {
        self.0.insert(
            "torrent_content_remove_option".into(),
            Value::String(v.into()),
        );
        self
    }
    pub fn memory_working_set_limit(mut self, v: i64) -> Self {
        self.0
            .insert("memory_working_set_limit".into(), Value::from(v));
        self
    }
    pub fn current_network_interface(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("current_network_interface".into(), Value::String(v.into()));
        self
    }
    pub fn current_interface_name(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("current_interface_name".into(), Value::String(v.into()));
        self
    }
    pub fn current_interface_address(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("current_interface_address".into(), Value::String(v.into()));
        self
    }
    pub fn save_resume_data_interval(mut self, v: i64) -> Self {
        self.0
            .insert("save_resume_data_interval".into(), Value::from(v));
        self
    }
    pub fn save_statistics_interval(mut self, v: i64) -> Self {
        self.0
            .insert("save_statistics_interval".into(), Value::from(v));
        self
    }
    pub fn torrent_file_size_limit(mut self, v: i64) -> Self {
        self.0
            .insert("torrent_file_size_limit".into(), Value::from(v));
        self
    }
    pub fn confirm_torrent_recheck(mut self, v: bool) -> Self {
        self.0
            .insert("confirm_torrent_recheck".into(), Value::from(v));
        self
    }
    pub fn recheck_completed_torrents(mut self, v: bool) -> Self {
        self.0
            .insert("recheck_completed_torrents".into(), Value::from(v));
        self
    }
    pub fn app_instance_name(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("app_instance_name".into(), Value::String(v.into()));
        self
    }
    pub fn refresh_interval(mut self, v: i64) -> Self {
        self.0.insert("refresh_interval".into(), Value::from(v));
        self
    }
    pub fn resolve_peer_host_names(mut self, v: bool) -> Self {
        self.0
            .insert("resolve_peer_host_names".into(), Value::from(v));
        self
    }
    pub fn resolve_peer_countries(mut self, v: bool) -> Self {
        self.0
            .insert("resolve_peer_countries".into(), Value::from(v));
        self
    }
    pub fn confirm_remove_all_tags(mut self, v: bool) -> Self {
        self.0
            .insert("confirm_remove_all_tags".into(), Value::from(v));
        self
    }
    pub fn confirm_remove_tracker_from_all_torrents(mut self, v: bool) -> Self {
        self.0.insert(
            "confirm_remove_tracker_from_all_torrents".into(),
            Value::from(v),
        );
        self
    }
    pub fn reannounce_when_address_changed(mut self, v: bool) -> Self {
        self.0
            .insert("reannounce_when_address_changed".into(), Value::from(v));
        self
    }
    pub fn enable_embedded_tracker(mut self, v: bool) -> Self {
        self.0
            .insert("enable_embedded_tracker".into(), Value::from(v));
        self
    }
    pub fn embedded_tracker_port(mut self, v: i64) -> Self {
        self.0
            .insert("embedded_tracker_port".into(), Value::from(v));
        self
    }
    pub fn embedded_tracker_port_forwarding(mut self, v: bool) -> Self {
        self.0
            .insert("embedded_tracker_port_forwarding".into(), Value::from(v));
        self
    }
    pub fn mark_of_the_web(mut self, v: bool) -> Self {
        self.0.insert("mark_of_the_web".into(), Value::from(v));
        self
    }
    pub fn ignore_ssl_errors(mut self, v: bool) -> Self {
        self.0.insert("ignore_ssl_errors".into(), Value::from(v));
        self
    }
    pub fn python_executable_path(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("python_executable_path".into(), Value::String(v.into()));
        self
    }
    pub fn start_paused(mut self, v: bool) -> Self {
        self.0.insert("start_paused".into(), Value::from(v));
        self
    }
    pub fn shutdown_timeout(mut self, v: i64) -> Self {
        self.0.insert("shutdown_timeout".into(), Value::from(v));
        self
    }
    pub fn bdecode_depth_limit(mut self, v: i64) -> Self {
        self.0.insert("bdecode_depth_limit".into(), Value::from(v));
        self
    }
    pub fn bdecode_token_limit(mut self, v: i64) -> Self {
        self.0.insert("bdecode_token_limit".into(), Value::from(v));
        self
    }
    pub fn load_torrent_max_directory_depth(mut self, v: i64) -> Self {
        self.0
            .insert("load_torrent_max_directory_depth".into(), Value::from(v));
        self
    }
    pub fn async_io_threads(mut self, v: i64) -> Self {
        self.0.insert("async_io_threads".into(), Value::from(v));
        self
    }
    pub fn hashing_threads(mut self, v: i64) -> Self {
        self.0.insert("hashing_threads".into(), Value::from(v));
        self
    }
    pub fn file_pool_size(mut self, v: i64) -> Self {
        self.0.insert("file_pool_size".into(), Value::from(v));
        self
    }
    pub fn checking_memory_use(mut self, v: i64) -> Self {
        self.0.insert("checking_memory_use".into(), Value::from(v));
        self
    }
    pub fn disk_cache(mut self, v: i64) -> Self {
        self.0.insert("disk_cache".into(), Value::from(v));
        self
    }
    pub fn disk_cache_ttl(mut self, v: i64) -> Self {
        self.0.insert("disk_cache_ttl".into(), Value::from(v));
        self
    }
    pub fn disk_queue_size(mut self, v: i64) -> Self {
        self.0.insert("disk_queue_size".into(), Value::from(v));
        self
    }
    pub fn disk_io_type(mut self, v: i64) -> Self {
        self.0.insert("disk_io_type".into(), Value::from(v));
        self
    }
    pub fn disk_io_read_mode(mut self, v: i64) -> Self {
        self.0.insert("disk_io_read_mode".into(), Value::from(v));
        self
    }
    pub fn disk_io_write_mode(mut self, v: i64) -> Self {
        self.0.insert("disk_io_write_mode".into(), Value::from(v));
        self
    }
    pub fn enable_coalesce_read_write(mut self, v: bool) -> Self {
        self.0
            .insert("enable_coalesce_read_write".into(), Value::from(v));
        self
    }
    pub fn enable_piece_extent_affinity(mut self, v: bool) -> Self {
        self.0
            .insert("enable_piece_extent_affinity".into(), Value::from(v));
        self
    }
    pub fn enable_upload_suggestions(mut self, v: bool) -> Self {
        self.0
            .insert("enable_upload_suggestions".into(), Value::from(v));
        self
    }
    pub fn send_buffer_watermark(mut self, v: i64) -> Self {
        self.0
            .insert("send_buffer_watermark".into(), Value::from(v));
        self
    }
    pub fn send_buffer_low_watermark(mut self, v: i64) -> Self {
        self.0
            .insert("send_buffer_low_watermark".into(), Value::from(v));
        self
    }
    pub fn send_buffer_watermark_factor(mut self, v: i64) -> Self {
        self.0
            .insert("send_buffer_watermark_factor".into(), Value::from(v));
        self
    }
    pub fn connection_speed(mut self, v: i64) -> Self {
        self.0.insert("connection_speed".into(), Value::from(v));
        self
    }
    pub fn seeding_outgoing_connections(mut self, v: bool) -> Self {
        self.0
            .insert("seeding_outgoing_connections".into(), Value::from(v));
        self
    }
    pub fn socket_send_buffer_size(mut self, v: i64) -> Self {
        self.0
            .insert("socket_send_buffer_size".into(), Value::from(v));
        self
    }
    pub fn socket_receive_buffer_size(mut self, v: i64) -> Self {
        self.0
            .insert("socket_receive_buffer_size".into(), Value::from(v));
        self
    }
    pub fn socket_backlog_size(mut self, v: i64) -> Self {
        self.0.insert("socket_backlog_size".into(), Value::from(v));
        self
    }
    pub fn outgoing_ports_min(mut self, v: i64) -> Self {
        self.0.insert("outgoing_ports_min".into(), Value::from(v));
        self
    }
    pub fn outgoing_ports_max(mut self, v: i64) -> Self {
        self.0.insert("outgoing_ports_max".into(), Value::from(v));
        self
    }
    pub fn upnp_lease_duration(mut self, v: i64) -> Self {
        self.0.insert("upnp_lease_duration".into(), Value::from(v));
        self
    }
    pub fn peer_tos(mut self, v: i64) -> Self {
        self.0.insert("peer_tos".into(), Value::from(v));
        self
    }
    pub fn utp_tcp_mixed_mode(mut self, v: i64) -> Self {
        self.0.insert("utp_tcp_mixed_mode".into(), Value::from(v));
        self
    }
    pub fn hostname_cache_ttl(mut self, v: i64) -> Self {
        self.0.insert("hostname_cache_ttl".into(), Value::from(v));
        self
    }
    pub fn idn_support_enabled(mut self, v: bool) -> Self {
        self.0.insert("idn_support_enabled".into(), Value::from(v));
        self
    }
    pub fn enable_multi_connections_from_same_ip(mut self, v: bool) -> Self {
        self.0.insert(
            "enable_multi_connections_from_same_ip".into(),
            Value::from(v),
        );
        self
    }
    pub fn enable_multi_connections_from_same_peer_id(mut self, v: bool) -> Self {
        self.0.insert(
            "enable_multi_connections_from_same_peer_id".into(),
            Value::from(v),
        );
        self
    }
    pub fn validate_https_tracker_certificate(mut self, v: bool) -> Self {
        self.0
            .insert("validate_https_tracker_certificate".into(), Value::from(v));
        self
    }
    pub fn ssrf_mitigation(mut self, v: bool) -> Self {
        self.0.insert("ssrf_mitigation".into(), Value::from(v));
        self
    }
    pub fn block_peers_on_privileged_ports(mut self, v: bool) -> Self {
        self.0
            .insert("block_peers_on_privileged_ports".into(), Value::from(v));
        self
    }
    pub fn upload_slots_behavior(mut self, v: i64) -> Self {
        self.0
            .insert("upload_slots_behavior".into(), Value::from(v));
        self
    }
    pub fn upload_choking_algorithm(mut self, v: i64) -> Self {
        self.0
            .insert("upload_choking_algorithm".into(), Value::from(v));
        self
    }
    pub fn announce_to_all_trackers(mut self, v: bool) -> Self {
        self.0
            .insert("announce_to_all_trackers".into(), Value::from(v));
        self
    }
    pub fn announce_to_all_tiers(mut self, v: bool) -> Self {
        self.0
            .insert("announce_to_all_tiers".into(), Value::from(v));
        self
    }
    pub fn announce_ip(mut self, v: impl Into<String>) -> Self {
        self.0.insert("announce_ip".into(), Value::String(v.into()));
        self
    }
    pub fn announce_port(mut self, v: i64) -> Self {
        self.0.insert("announce_port".into(), Value::from(v));
        self
    }
    pub fn max_concurrent_http_announces(mut self, v: i64) -> Self {
        self.0
            .insert("max_concurrent_http_announces".into(), Value::from(v));
        self
    }
    pub fn stop_tracker_timeout(mut self, v: i64) -> Self {
        self.0.insert("stop_tracker_timeout".into(), Value::from(v));
        self
    }
    pub fn peer_turnover(mut self, v: i64) -> Self {
        self.0.insert("peer_turnover".into(), Value::from(v));
        self
    }
    pub fn peer_turnover_cutoff(mut self, v: i64) -> Self {
        self.0.insert("peer_turnover_cutoff".into(), Value::from(v));
        self
    }
    pub fn peer_turnover_interval(mut self, v: i64) -> Self {
        self.0
            .insert("peer_turnover_interval".into(), Value::from(v));
        self
    }
    pub fn request_queue_size(mut self, v: i64) -> Self {
        self.0.insert("request_queue_size".into(), Value::from(v));
        self
    }
    pub fn max_outstanding_block_requests(mut self, v: i64) -> Self {
        self.0
            .insert("max_outstanding_block_requests".into(), Value::from(v));
        self
    }
    pub fn dht_bootstrap_nodes(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("dht_bootstrap_nodes".into(), Value::String(v.into()));
        self
    }
    pub fn webtorrent_stun_server(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("webtorrent_stun_server".into(), Value::String(v.into()));
        self
    }
    /// Write-only: the Web UI password (never returned by the server).
    pub fn web_ui_password(mut self, v: impl Into<String>) -> Self {
        self.0
            .insert("web_ui_password".into(), Value::String(v.into()));
        self
    }
}

impl fmt::Debug for PreferencesPatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut m = f.debug_map();
        for (k, v) in &self.0 {
            if crate::models::app::is_secret_key(k) {
                m.entry(k, &"<redacted>");
            } else {
                m.entry(k, v);
            }
        }
        m.finish()
    }
}

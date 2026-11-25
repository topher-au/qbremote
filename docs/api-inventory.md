# qBittorrent WebUI API inventory

This file is the ground truth for what the client must implement. I took it from qBittorrent's **source code**, not the wiki, because the wiki is stale.

- Source: `qbittorrent/qBittorrent` master, commit `e6b69eb4b532146b34177b405734d36ff7624762` (2026-11-23).
- `API_VERSION` in `src/webui/webapplication.h` is **2.17.0**.
- Files read: `src/webui/webapplication.{h,cpp}`, `src/webui/api/*controller.{h,cpp}`, `src/webui/api/serialize/serialize_torrent.{h,cpp}` and `WebAPI_Changelog.md`.

Where the wiki ("WebUI API (qBittorrent 5.0)") disagrees with the source (confirmed examples):

| Topic | Wiki says | Source says |
|---|---|---|
| Stopped torrent states | `pausedUP`, `pausedDL` | `stoppedUP`, `stoppedDL` |
| Other states | lists `allocating` | no `allocating`; adds `forcedMetaDL` |
| Private flag (`torrents/info`) | `isPrivate` | `private` (only when metadata is present) |
| Private flag (`torrents/properties`) | `isPrivate` | `private`; `is_private` is kept but deprecated |
| `torrents/add` "paused" | `paused`, `root_folder`, `skip_checking` | `stopped`, `contentLayout`; `skip_checking` was removed in 2.16.0 (use `seedMode`) |
| Undocumented endpoints | — | `torrents/count`, `addWebSeeds`, `editWebSeed`, `removeWebSeeds`, `setTags`, `setSavePath`, `setDownloadPath`, `export`, `SSLParameters`, `setSSLParameters`, `app/sendTestEmail`, `app/networkInterfaceList`, `app/networkInterfaceAddressList`, `rss/setFeedURL`, `transfer/setSpeedLimitsMode`, the whole `torrentcreator/*` and `clientdata/*` namespaces |

**Rule for implementers:** before coding an endpoint, read its `…Action()` body in the pinned source. This table is an index. It does not replace the source.

## Transport rules (from `webapplication.cpp`)

- Base path is `/api/v2/<scope>/<action>`. The scopes are `app`, `auth`, `clientdata`, `log`, `rss`, `search`, `sync`, `torrentcreator`, `torrents` and `transfer`.
- **Methods:** actions listed in `m_allowedMethod` (marked **P** below) accept **POST only**. Every other action accepts GET, POST or HEAD. A wrong method returns **405**.
- **Parameters:** GET and HEAD read the query string. POST reads the form body (`application/x-www-form-urlencoded` or `multipart/form-data`). File uploads (`torrents/add`, `torrents/parseMetadata`, `rss/importRules`) use multipart, and the server reads the parts as `data()`.
- **Error mapping** (`APIErrorType` → HTTP): AccessDenied → 403, BadData → 415, BadParams → 400, Conflict → 409, NotFound → 404, Unauthorized → 401. `APIStatus::Async` → **202**. Since 2.11.8 some empty responses return 204 instead of 200.
- **Session cookie:** the name is `QBT_SID_<server's configured WebUI port>`. Behind a reverse proxy this port is not the port the client connects to, so the client must keep whatever `Set-Cookie` returns and never build the cookie name itself.
- **CSRF:** the server rejects only requests it classifies as *cross-origin* from `Sec-Fetch-Site`, `Origin` or `Referer`. A native client that sends none of these headers passes. The client must **never** send `Origin`, `Referer` or `Sec-Fetch-*`.
- **Host-header validation:** when enabled, `Host` must match `web_ui_domain_list` or the server's address. Show this as a specific error message to the user.
- **API-key auth** (2.14.1+): `Authorization: Bearer <key>` or `X-API-Key: <key>`. CSRF checks are skipped for API-key requests. An invalid key returns 401 with `WWW-Authenticate: Bearer` (2.17.0+).
- **Basic auth** (2.15.0+): `Authorization: Basic …` starts a session. Since 2.17.0, Basic auth combined with a stale session cookie returns 403 on API paths other than `auth/login`. The client must clear its cookie and log in again.
- **Login:** `auth/login` returns 401 for bad credentials (2.14.0+). It returns 403 while the IP is banned (after `web_ui_max_auth_fail_count` failures, for `web_ui_ban_duration` seconds).

## Endpoints

Legend: **P** = POST-only. "→" = response shape. Lists separated by `|` use that character literally. `hashes` accepts `all` where noted.

### auth
| Action | M | Params | Notes |
|---|---|---|---|
| login | P | `username`, `password` | Sets the session cookie. 401 bad creds, 403 banned |
| logout | P | — | |

### app
| Action | M | Params | Notes |
|---|---|---|---|
| version | | — | text, e.g. `v5.x.y` |
| webapiVersion | | — | text, e.g. `2.17.0` |
| buildInfo | | — | `{qt, libtorrent, boost, openssl, zlib, bitness, platform}`; `platform` is linux/macos/windows/unknown |
| processInfo | | — | `{launch_time}` (2.15.1+) |
| shutdown | P | — | Confirm before calling |
| preferences | | — | object; see the key list below |
| setPreferences | P | `json` (JSON object holding only the changed keys) | |
| defaultSavePath | | — | text |
| sendTestEmail | P | — | |
| getDirectoryContent | | `dirPath`, `mode` (`all`/`dirs`/`files`, default `all`), `withMetadata` | Remote file browser. Entries are `{name,type(dir/file),size,creation_date,last_access_date,last_modification_date}` |
| getFreeSpaceAtPath | | `path` | |
| cookies | | — | `[{name,domain,path,value,expirationDate}]` |
| setCookies | P | `cookies` (JSON array) | |
| rotateAPIKey | P | — | Returns the new key. Show it once and never log it |
| deleteAPIKey | P | — | |
| networkInterfaceList | | — | Used by the Advanced options page |
| networkInterfaceAddressList | | `iface` | |

### log
| Action | M | Params |
|---|---|---|
| main | | `normal`, `info`, `warning`, `critical` (bools), `last_known_id` → `[{id,message,timestamp,type}]`; type is 1/2/4/8 |
| peers | | `last_known_id` → `[{id,ip,timestamp,blocked,reason}]` |

### sync
| Action | M | Params | Notes |
|---|---|---|---|
| maindata | | `rid` | Keys: `rid`, `full_update`, `torrents{hash→partial}`, `torrents_removed`, `categories{name→partial}`, `categories_removed`, `tags[]`, `tags_removed`, `trackers{url→[hash]}`, `trackers_removed`, `server_state{…}` |
| torrentPeers | | `hash`, `rid` | Keys: `rid`, `full_update`, `peers{"ip:port"→partial}`, `peers_removed`, `show_flags` |

`server_state` keys: `alltime_dl`, `alltime_ul`, `average_time_queue`, `connection_status`, `dht_nodes`, `dl_info_data`, `dl_info_speed`, `dl_rate_limit`, `free_space_on_disk`, `global_ratio`, `last_external_address_v4`, `last_external_address_v6`, `queued_io_jobs`, `queued_tracker_announces`, `queueing`, `read_cache_hits`, `read_cache_overload`, `refresh_interval`, `request_latency`, `session_state`, `total_buffers_size`, `total_peer_connections`, `total_queued_size`, `total_wasted_session`, `up_info_data`, `up_info_speed`, `up_rate_limit`, `use_alt_speed_limits`, `write_cache_overload`.

Torrent entries in maindata also carry `has_tracker_warning`, `has_tracker_error` and `has_other_announce_error`.

Peer keys: `client`, `peer_id_client`, `connection`, `country`, `country_code`, `dl_speed`, `up_speed`, `downloaded`, `uploaded`, `files`, `flags`, `flags_desc`, `host_name`, `ip`, `port`, `i2p_dest` (I2P peers have no ip/port), `progress`, `relevance`, `contribution`.

### transfer
| Action | M | Params |
|---|---|---|
| info | | → `dl_info_speed, dl_info_data, dl_rate_limit, up_info_speed, up_info_data, up_rate_limit, dht_nodes, connection_status, last_external_address_v4, last_external_address_v6` |
| speedLimitsMode | | → `0`/`1` |
| setSpeedLimitsMode | P | `mode` |
| toggleSpeedLimitsMode | P | — |
| downloadLimit / uploadLimit | | → B/s |
| setDownloadLimit / setUploadLimit | P | `limit` (B/s) |
| getSpeedLimits | | → `{up_limit, dl_limit, alt_up_limit, alt_dl_limit}` |
| setSpeedLimits | P | `up_limit`, `dl_limit`, `alt_up_limit`, `alt_dl_limit` |
| banPeers | P | `peers` (`host:port\|…`) |
| pauseSession / resumeSession | P | — (whole BitTorrent session; `server_state.session_state`) |

### torrents
| Action | M | Params |
|---|---|---|
| count | | → integer text |
| info | | `filter`, `category`, `tag`, `sort`, `reverse`, `limit`, `offset`, `hashes`, `private`, `includeFiles`, `includeTrackers` |
| properties | | `hash` |
| trackers | | `hash` |
| webseeds | | `hash` → `[{url}]` |
| addWebSeeds | P | `hash`, `urls` |
| editWebSeed | P | `hash`, `origUrl`, `newUrl` |
| removeWebSeeds | P | `hash`, `urls` |
| files | | `hash`, `indexes` |
| pieceHashes / pieceStates / pieceAvailability | | `hash` |
| start / stop | P | `hashes` (or `all`) |
| recheck / reannounce | P | `hashes`; reannounce also takes optional `urls` |
| delete | P | `hashes`, `deleteFiles` |
| rename | P | `hash`, `name` |
| setComment | P | `hashes`, `comment` |
| setCategory | P | `hashes`, `category` |
| categories | | → `{name→{name,savePath,downloadPath,…share limits}}` |
| createCategory / editCategory | P | `category`, `savePath`, `downloadPathEnabled`, `downloadPath`, `ratioLimit`, `seedingTimeLimit`, `inactiveSeedingTimeLimit`, `shareLimitsMode`, `shareLimitAction` |
| removeCategories | P | `categories` (newline-separated) |
| tags | | → `[string]` |
| addTags / setTags / removeTags | P | `hashes`, `tags` (comma-separated) |
| createTags / deleteTags | P | `tags` |
| add | P | multipart: `urls` (newline-separated), torrent file parts, `savepath`, `downloadPath`, `useDownloadPath`, `category`, `tags`, `rename`, `upLimit`, `dlLimit`, `ratioLimit`, `seedingTimeLimit`, `inactiveSeedingTimeLimit`, `shareLimitsMode`, `shareLimitAction`, `autoTMM`, `stopped`, `forced`, `addToTopOfQueue`, `stopCondition`, `contentLayout`, `seedMode`, `sequentialDownload`, `firstLastPiecePrio`, `filePriorities` (comma-separated, single source only, requires metadata), `downloader`, `ssl_certificate`, `ssl_private_key`, `ssl_dh_params` → `{success_count, failure_count, pending_count, added_torrent_ids}`; 202 if pending, 409 if nothing added, 415 invalid file |
| addTrackers | P | `hash`, `urls` (newline-separated) |
| editTracker | P | `hash`, `url`, `newUrl`?, `tier`? |
| removeTrackers | P | `hash`, `urls` (`\|`) |
| addPeers | P | `hashes`, `peers` |
| filePrio | P | `hash`, `id` (`\|` indexes), `priority` (0/1/6/7) |
| uploadLimit / downloadLimit | | `hashes` → `{hash→B/s}` |
| setUploadLimit / setDownloadLimit | P | `hashes`, `limit` |
| setShareLimits | P | `hashes`, `ratioLimit`, `seedingTimeLimit`, `inactiveSeedingTimeLimit`, `shareLimitsMode`, `shareLimitAction` |
| toggleSequentialDownload / toggleFirstLastPiecePrio | P | `hashes` |
| setSuperSeeding / setForceStart | P | `hashes`, `value` |
| increasePrio / decreasePrio / topPrio / bottomPrio | P | `hashes` (409 if queueing is disabled) |
| setLocation | P | `hashes`, `location` |
| setSavePath / setDownloadPath | P | `id` (hashes), `path` |
| setAutoManagement | P | `hashes`, `enable` |
| renameFile / renameFolder | P | `hash`, `oldPath`, `newPath` |
| export | | `hash` → `application/x-bittorrent` file |
| SSLParameters | | `hash` |
| setSSLParameters | P | `hash`, `ssl_certificate`, `ssl_private_key`, `ssl_dh_params` |
| fetchMetadata | P | `source` (URL or magnet), `downloader`?; call again to poll |
| cancelFetchMetadata | P | `source` |
| parseMetadata | P | multipart torrent files → array in request order |
| saveMetadata | | `source` → .torrent file |
| downloadFile | | `hash`, `file` (index or relative path) → file bytes (2.16.0+) |

`filter` values: `all`, `downloading`, `seeding`, `completed`, `stopped`, `running`, `active`, `inactive`, `stalled`, `stalled_uploading`, `stalled_downloading`, `checking`, `moving`, `errored`.

`state` values: `error`, `missingFiles`, `uploading`, `stoppedUP`, `queuedUP`, `stalledUP`, `checkingUP`, `forcedUP`, `downloading`, `metaDL`, `forcedMetaDL`, `stoppedDL`, `queuedDL`, `stalledDL`, `checkingDL`, `forcedDL`, `checkingResumeData`, `moving`, `unknown`.

Torrent object keys (`serialize_torrent.h`): `hash`, `infohash_v1`, `infohash_v2`, `name`, `magnet_uri`, `size`, `progress`, `dlspeed`, `upspeed`, `priority`, `num_seeds`, `num_complete`, `num_leechs`, `num_incomplete`, `ratio`, `popularity`, `eta`, `state`, `error`, `seq_dl`, `f_l_piece_prio`, `category`, `tags`, `super_seeding`, `force_start`, `save_path`, `download_path`, `content_path`, `root_path`, `added_on`, `completion_on`, `tracker`, `trackers_count`, `dl_limit`, `up_limit`, `downloaded`, `uploaded`, `downloaded_session`, `uploaded_session`, `amount_left`, `completed`, `max_ratio`, `max_seeding_time`, `max_inactive_seeding_time`, `ratio_limit`, `seeding_time_limit`, `inactive_seeding_time_limit`, `share_limits_mode`, `share_limit_action`, `seen_complete`, `last_activity`, `total_size`, `auto_tmm`, `time_active`, `seeding_time`, `availability`, `reannounce`, `comment`, `private`, `has_metadata`, `connections_count`, `connections_limit`, `total_wasted`, `pieces_num`, `piece_size`, `pieces_have`, `created_by`, `creation_date`.

Tracker keys: `url`, `status`, `tier`, `msg`, `num_peers`, `num_seeds`, `num_leeches`, `num_downloaded`, `next_announce`, `min_announce`, `disabled_for_torrent`, `endpoints[{name, updating, status, msg, bt_version, num_peers, num_seeds, num_leeches, num_downloaded, next_announce, min_announce}]`. Status values: 0 disabled, 1 not contacted, 2 working, 3 updating, 4 not working, 5 tracker error, 6 unreachable.

### torrentcreator
| Action | M | Params |
|---|---|---|
| addTask | P | `sourcePath` (remote path), `torrentFilePath`?, `format` (v1/v2/hybrid), `pieceSize`, `optimizeAlignment`, `paddedFileSizeLimit`, `private`, `startSeeding`, `comment`, `source`, `trackers`, `urlSeeds`, `ignoreDotfiles` → `{taskID}` |
| status | | `taskID`? → `[{taskID, status (Queued/Running/Finished/Failed), progress, errorMessage, timeAdded, timeStarted, timeFinished, …the addTask options}]` |
| torrentFile | | `taskID` → .torrent bytes |
| deleteTask | P | `taskID` |

### rss
| Action | M | Params |
|---|---|---|
| addFolder | P | `path` |
| addFeed | P | `url`, `path`, `refreshInterval`? |
| setFeedURL | P | `path`, `url` |
| setFeedRefreshInterval | P | `path`, `refreshInterval` |
| removeItem | P | `path` |
| moveItem | P | `itemPath`, `destPath` |
| items | | `withData` |
| markAsRead | P | `itemPath`, `articleId`? |
| refreshItem | P | `itemPath` |
| setRule | P | `ruleName`, `ruleDef` (JSON) |
| renameRule | P | `ruleName`, `newRuleName` |
| cloneRule | P | `sourceName`, `cloneName` |
| removeRule | P | `ruleName` |
| rules | | — |
| matchingArticles | | `ruleName` |
| exportRules | | — |
| importRules | P | multipart file |

### search
| Action | M | Params |
|---|---|---|
| start | P | `pattern`, `category`, `plugins` → `{id}`; 409 when the concurrent-search limit is reached |
| stop / delete | P | `id` |
| status | | `id`? |
| results | | `id`, `limit`, `offset` → `{results[{fileName, fileUrl, fileSize, nbSeeders, nbLeechers, engineName, engineFullName, siteUrl, descrLink, pubDate}], status, total}` |
| downloadTorrent | P | `torrentUrl`, `pluginName` |
| plugins | | — |
| installPlugin | P | `sources` (`\|` URLs or **server-side** paths) |
| uninstallPlugin | P | `names` |
| enablePlugin | P | `names`, `enable` |
| updatePlugins | P | — |

### clientdata
| Action | M | Params |
|---|---|---|
| load | | `keys` (JSON array) |
| store | P | `data` (JSON object) |

## app/preferences keys (243, in source order, grouped by the source's section comments)

**Behavior:** `locale performance_warning status_bar_external_ip confirm_torrent_deletion store_search_jobs store_search_job_results file_log_enabled file_log_path file_log_backup_enabled file_log_max_size file_log_delete_old file_log_age file_log_age_type delete_torrent_content_files`

**Downloads:** `torrent_content_layout add_to_top_of_queue add_stopped_enabled torrent_stop_condition merge_trackers auto_delete_mode preallocate_all incomplete_files_ext use_unwanted_folder auto_tmm_enabled torrent_changed_tmm_enabled save_path_changed_tmm_enabled category_changed_tmm_enabled save_path temp_path_enabled temp_path use_category_paths_in_manual_mode torrent_files_backup_enabled torrent_files_backup_dir torrent_files_finished_backup_dir_enabled torrent_files_finished_backup_dir remove_torrent_file_backup scan_dirs excluded_file_names_enabled excluded_file_names exclude_files_without_extension mail_notification_enabled mail_notification_sender mail_notification_email mail_notification_smtp mail_notification_encryption_type mail_notification_auth_enabled mail_notification_username mail_notification_password autorun_on_torrent_added_enabled autorun_on_torrent_added_program autorun_enabled autorun_program`

**Connection:** `listen_port ssl_enabled ssl_listen_port random_port upnp max_connec max_connec_per_torrent max_uploads max_uploads_per_torrent i2p_enabled i2p_address i2p_port i2p_mixed_mode i2p_pex_enabled i2p_inbound_quantity i2p_outbound_quantity i2p_inbound_length i2p_outbound_length i2p_inbound_length_variance i2p_outbound_length_variance proxy_type proxy_ip proxy_port proxy_auth_enabled proxy_username proxy_password proxy_hostname_lookup proxy_bittorrent proxy_peer_connections proxy_rss proxy_misc ip_filter_enabled ip_filter_path ip_filter_trackers banned_IPs`

**Speed:** `dl_limit up_limit alt_dl_limit alt_up_limit bittorrent_protocol limit_utp_rate limit_tcp_overhead limit_lan_peers scheduler_enabled schedule_from_hour schedule_from_min schedule_to_hour schedule_to_min scheduler_days`

**BitTorrent:** `dht pex lsd encryption anonymous_mode max_active_checking_torrents queueing_enabled max_active_downloads max_active_torrents max_active_uploads dont_count_slow_torrents slow_torrent_dl_rate_threshold slow_torrent_ul_rate_threshold slow_torrent_inactive_timer max_ratio_enabled max_ratio max_seeding_time_enabled max_seeding_time max_inactive_seeding_time_enabled max_inactive_seeding_time share_limits_mode max_ratio_act add_trackers_enabled add_trackers add_trackers_from_url_enabled add_trackers_url add_trackers_url_list`

**WebUI:** `web_ui_domain_list web_ui_address web_ui_port web_ui_upnp use_https web_ui_https_cert_path web_ui_https_key_path web_ui_username bypass_local_auth bypass_auth_subnet_whitelist_enabled bypass_auth_subnet_whitelist web_ui_max_auth_fail_count web_ui_ban_duration web_ui_session_timeout web_ui_sessions_count_limit web_ui_api_key alternative_webui_enabled alternative_webui_path web_ui_clickjacking_protection_enabled web_ui_csrf_protection_enabled web_ui_secure_cookie_enabled web_ui_host_header_validation_enabled web_ui_use_custom_http_headers_enabled web_ui_custom_http_headers web_ui_reverse_proxy_enabled web_ui_reverse_proxies_list dyndns_enabled dyndns_service dyndns_username dyndns_password dyndns_domain`

The WebUI page also accepts the **write-only** `web_ui_password`, which is never returned.

**RSS:** `rss_refresh_interval rss_fetch_delay rss_max_articles_per_feed rss_processing_enabled rss_auto_downloading_enabled rss_download_repack_proper_episodes rss_smart_episode_filters`

**Advanced:** `resume_data_storage_type torrent_content_remove_option memory_working_set_limit current_network_interface current_interface_name current_interface_address save_resume_data_interval save_statistics_interval torrent_file_size_limit confirm_torrent_recheck recheck_completed_torrents app_instance_name refresh_interval resolve_peer_host_names resolve_peer_countries confirm_remove_all_tags confirm_remove_tracker_from_all_torrents reannounce_when_address_changed enable_embedded_tracker embedded_tracker_port embedded_tracker_port_forwarding mark_of_the_web ignore_ssl_errors python_executable_path start_paused shutdown_timeout bdecode_depth_limit bdecode_token_limit load_torrent_max_directory_depth async_io_threads hashing_threads file_pool_size checking_memory_use disk_cache disk_cache_ttl disk_queue_size disk_io_type disk_io_read_mode disk_io_write_mode enable_coalesce_read_write enable_piece_extent_affinity enable_upload_suggestions send_buffer_watermark send_buffer_low_watermark send_buffer_watermark_factor connection_speed seeding_outgoing_connections socket_send_buffer_size socket_receive_buffer_size socket_backlog_size outgoing_ports_min outgoing_ports_max upnp_lease_duration peer_tos utp_tcp_mixed_mode hostname_cache_ttl idn_support_enabled enable_multi_connections_from_same_ip enable_multi_connections_from_same_peer_id validate_https_tracker_certificate ssrf_mitigation block_peers_on_privileged_ports upload_slots_behavior upload_choking_algorithm announce_to_all_trackers announce_to_all_tiers announce_ip announce_port max_concurrent_http_announces stop_tracker_timeout peer_turnover peer_turnover_cutoff peer_turnover_interval request_queue_size max_outstanding_block_requests dht_bootstrap_nodes webtorrent_stun_server`

Keys that contain secrets (`*_password`, `web_ui_api_key`, `proxy_password`, `dyndns_password`, `mail_notification_password`) must be redacted in every log and debug view.

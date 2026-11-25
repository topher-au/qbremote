use super::lenient;

lenient!(TransferInfo {
    dl_info_speed: i64,
    dl_info_data: i64,
    dl_rate_limit: i64,
    up_info_speed: i64,
    up_info_data: i64,
    up_rate_limit: i64,
    dht_nodes: i64,
    connection_status: String,
    last_external_address_v4: String,
    last_external_address_v6: String,
});

lenient!(SpeedLimits {
    up_limit: i64,
    dl_limit: i64,
    alt_up_limit: i64,
    alt_dl_limit: i64
});

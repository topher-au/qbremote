use super::lenient;

lenient!(SearchResult {
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "fileUrl")]
    file_url: String,
    #[serde(rename = "fileSize")]
    file_size: i64,
    #[serde(rename = "nbSeeders")]
    nb_seeders: i64,
    #[serde(rename = "nbLeechers")]
    nb_leechers: i64,
    #[serde(rename = "engineName")]
    engine_name: String,
    #[serde(rename = "engineFullName")]
    engine_full_name: String,
    #[serde(rename = "siteUrl")]
    site_url: String,
    #[serde(rename = "descrLink")]
    descr_link: String,
    #[serde(rename = "pubDate")]
    pub_date: i64,
});

lenient!(SearchResults { results: Vec<SearchResult>, status: String, total: i64 });
lenient!(SearchStatus {
    id: i64,
    status: String,
    total: i64
});
lenient!(SearchStart { id: i64 });
lenient!(SearchCategory {
    id: String,
    name: String
});
lenient!(SearchPlugin {
    enabled: bool, #[serde(rename = "fullName")] full_name: String, name: String,
    #[serde(rename = "supportedCategories")] supported_categories: Vec<SearchCategory>,
    url: String, version: String,
});

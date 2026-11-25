//! Typed request-parameter encoding: the server's list separators differ per action.

/// Ordered form/query parameters.
#[derive(Debug, Default, Clone)]
pub struct Params(pub(crate) Vec<(String, String)>);

impl Params {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn str(mut self, k: &str, v: impl Into<String>) -> Self {
        self.0.push((k.to_owned(), v.into()));
        self
    }
    /// Booleans are sent as `true`/`false`.
    pub fn bool(self, k: &str, v: bool) -> Self {
        self.str(k, if v { "true" } else { "false" })
    }
    pub fn int(self, k: &str, v: impl ToString) -> Self {
        self.str(k, v.to_string())
    }
    pub fn opt_str(self, k: &str, v: Option<&str>) -> Self {
        match v {
            Some(v) => self.str(k, v),
            None => self,
        }
    }
    pub fn hashes(self, k: &str, h: &Hashes) -> Self {
        self.str(k, h.encode())
    }
    /// Tags are comma-separated.
    pub fn tags(self, k: &str, tags: &[String]) -> Self {
        self.str(k, tags.join(","))
    }
    /// URLs and category names are newline-separated.
    pub fn lines(self, k: &str, items: &[String]) -> Self {
        self.str(k, items.join("\n"))
    }
    /// `|`-separated lists (file ids, tracker URLs for removal, peers).
    pub fn pipe(self, k: &str, items: &[String]) -> Self {
        self.str(k, items.join("|"))
    }
    pub fn pairs(&self) -> &[(String, String)] {
        &self.0
    }
}

/// A set of torrent hashes, or every torrent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hashes {
    All,
    Some(Vec<String>),
}

impl Hashes {
    pub fn one(h: impl Into<String>) -> Self {
        Self::Some(vec![h.into()])
    }
    pub fn encode(&self) -> String {
        match self {
            Self::All => "all".to_owned(),
            Self::Some(v) => v.join("|"),
        }
    }
}

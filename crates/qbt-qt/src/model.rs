//! Item models exposed to QML. All `begin*`/`end*` pairs go through the `ModelMutator` helpers below.

use core::pin::Pin;
use std::{collections::HashMap, sync::Mutex};

use cxx_qt::{CxxQtThread, CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QList, QModelIndex, QString, QVariant,
};
use qbt_sync::{
    MainChanges, RowOp, ViewIndex, ViewQuery,
    filter::{CategoryFilter, StatusFilter, TagFilter, TrackerFilter},
};
use serde_json::Value;

use crate::{
    columns, state,
    tables::{Payload, Row},
};

pub const ROLE_DISPLAY: i32 = 0;
pub const ROLE_VALUE: i32 = 256;
pub const ROLE_KEY: i32 = 257;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;

        include!(<QtCore/QAbstractTableModel>);
        type QAbstractTableModel;
    }

    // ---------------------------------------------------------------- GenericTable
    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractTableModel]
        type GenericTable = super::GenericTableRust;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &GenericTable, parent: &QModelIndex) -> i32;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "columnCount"]
        fn column_count(self: &GenericTable, parent: &QModelIndex) -> i32;
        #[qinvokable]
        #[cxx_override]
        fn data(self: &GenericTable, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &GenericTable) -> QHash_i32_QByteArray;

        /// Bind to a data source and fix the column count. Call once from `Component.onCompleted`.
        #[qinvokable]
        fn configure(self: Pin<&mut GenericTable>, source: &QString, columns: i32);
        /// Number of rows (QML cannot call `rowCount()` without a parent index).
        #[qinvokable]
        fn count(self: &GenericTable) -> i32;
        #[qinvokable]
        fn key_at(self: &GenericTable, row: i32) -> QString;
        #[qinvokable]
        fn row_of_key(self: &GenericTable, key: &QString) -> i32;
        #[qinvokable]
        fn cell(self: &GenericTable, row: i32, col: i32) -> QVariant;
        #[qinvokable]
        fn sort_by(self: Pin<&mut GenericTable>, col: i32, descending: bool);
        #[qinvokable]
        fn set_filter(self: Pin<&mut GenericTable>, text: &QString);

        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut GenericTable>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut GenericTable>);
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(
            self: Pin<&mut GenericTable>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut GenericTable>);
        #[inherit]
        fn index(self: &GenericTable, row: i32, column: i32, parent: &QModelIndex) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut GenericTable>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QList_i32,
        );
    }
    impl cxx_qt::Threading for GenericTable {}

    // ---------------------------------------------------------------- TransferListModel
    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractTableModel]
        type TransferListModel = super::TransferListModelRust;

        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &TransferListModel, parent: &QModelIndex) -> i32;
        #[qinvokable]
        #[cxx_override]
        #[cxx_name = "columnCount"]
        fn column_count(self: &TransferListModel, parent: &QModelIndex) -> i32;
        #[qinvokable]
        #[cxx_override]
        fn data(self: &TransferListModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &TransferListModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn header(self: &TransferListModel, col: i32) -> QString;
        #[qinvokable]
        fn column_key(self: &TransferListModel, col: i32) -> QString;
        #[qinvokable]
        fn column_default_visible(self: &TransferListModel, col: i32) -> bool;
        #[qinvokable]
        fn hash_at(self: &TransferListModel, row: i32) -> QString;
        #[qinvokable]
        fn row_of_hash(self: &TransferListModel, hash: &QString) -> i32;
        /// Display text of a torrent field (`key` as in the torrent object), for copy/rename.
        #[qinvokable]
        fn text_at(self: &TransferListModel, row: i32, key: &QString) -> QString;
        /// Raw JSON field value as text; booleans and numbers use their JSON form.
        #[qinvokable]
        fn raw_at(self: &TransferListModel, row: i32, key: &QString) -> QString;
        /// `kind`: status, category, tag or tracker. See `tables::sidebar` for key conventions.
        #[qinvokable]
        fn set_filter(self: Pin<&mut TransferListModel>, kind: &QString, key: &QString);
        #[qinvokable]
        fn set_search(self: Pin<&mut TransferListModel>, text: &QString, regex: bool);
        #[qinvokable]
        fn sort_by(self: Pin<&mut TransferListModel>, col: i32, descending: bool);
        #[qinvokable]
        fn sort_column(self: &TransferListModel) -> i32;
        /// The view reports which rows are on screen so updates only signal those (plus a margin).
        #[qinvokable]
        fn set_visible_rows(self: Pin<&mut TransferListModel>, first: i32, last: i32);
        #[qinvokable]
        fn sort_descending(self: &TransferListModel) -> bool;

        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut TransferListModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut TransferListModel>);
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(
            self: Pin<&mut TransferListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut TransferListModel>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        unsafe fn begin_remove_rows(
            self: Pin<&mut TransferListModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        unsafe fn end_remove_rows(self: Pin<&mut TransferListModel>);
        #[inherit]
        #[cxx_name = "beginMoveRows"]
        unsafe fn begin_move_rows(
            self: Pin<&mut TransferListModel>,
            src_parent: &QModelIndex,
            first: i32,
            last: i32,
            dest_parent: &QModelIndex,
            dest_child: i32,
        ) -> bool;
        #[inherit]
        #[cxx_name = "endMoveRows"]
        unsafe fn end_move_rows(self: Pin<&mut TransferListModel>);
        #[inherit]
        fn index(
            self: &TransferListModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
        #[inherit]
        #[qsignal]
        #[cxx_name = "dataChanged"]
        fn data_changed(
            self: Pin<&mut TransferListModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QList_i32,
        );
    }
    impl cxx_qt::Threading for TransferListModel {}
    impl cxx_qt::Initialize for TransferListModel {}
}

// ---------------------------------------------------------------------------------------------

fn to_variant(v: &Value) -> QVariant {
    match v {
        Value::Null => QVariant::default(),
        Value::Bool(b) => QVariant::from(b),
        Value::Number(n) => match n.as_i64() {
            Some(i) => QVariant::from(&i),
            None => QVariant::from(&n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => QVariant::from(&QString::from(s.as_str())),
        other => QVariant::from(&QString::from(other.to_string().as_str())),
    }
}

fn to_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn role_names() -> QHash<QHashPair_i32_QByteArray> {
    let mut h = QHash::<QHashPair_i32_QByteArray>::default();
    h.insert(ROLE_DISPLAY, QByteArray::from("display"));
    h.insert(ROLE_VALUE, QByteArray::from("value"));
    h.insert(ROLE_KEY, QByteArray::from("key"));
    h
}

fn no_parent() -> QModelIndex {
    QModelIndex::default()
}

/// Wraps every `begin*`/`end*` pair so a pair can never be left open. The `unsafe` is confined here.
macro_rules! impl_mutator {
    ($t:ty) => {
        #[allow(dead_code)]
        impl $t {
            fn mutate_insert(
                mut self: Pin<&mut Self>,
                first: i32,
                last: i32,
                f: impl FnOnce(Pin<&mut Self>),
            ) {
                // SAFETY: paired with `end_insert_rows` below; `f` only edits Rust-side storage.
                unsafe { self.as_mut().begin_insert_rows(&no_parent(), first, last) };
                f(self.as_mut());
                unsafe { self.as_mut().end_insert_rows() };
            }
            fn mutate_reset(mut self: Pin<&mut Self>, f: impl FnOnce(Pin<&mut Self>)) {
                // SAFETY: paired with `end_reset_model` below.
                unsafe { self.as_mut().begin_reset_model() };
                f(self.as_mut());
                unsafe { self.as_mut().end_reset_model() };
            }
            fn emit_rows_changed(self: Pin<&mut Self>, first: i32, last: i32, cols: i32) {
                self.emit_cells_changed(first, last, 0, cols - 1);
            }
            // dataChanged for rows first..=last and columns col_first..=col_last.
            fn emit_cells_changed(
                mut self: Pin<&mut Self>,
                first: i32,
                last: i32,
                col_first: i32,
                col_last: i32,
            ) {
                if col_last < col_first {
                    return;
                }
                let tl = self.as_ref().index(first, col_first, &no_parent());
                let br = self.as_ref().index(last, col_last, &no_parent());
                self.as_mut()
                    .data_changed(&tl, &br, &QList::<i32>::default());
            }
        }
    };
}
impl_mutator!(qobject::GenericTable);
impl_mutator!(qobject::TransferListModel);

// =========================================================================== GenericTable

type TableThread = CxxQtThread<qobject::GenericTable>;
static TABLES: Mutex<Option<HashMap<String, Vec<TableThread>>>> = Mutex::new(None);

/// Deliver data to every `GenericTable` bound to `source`. Callable from any thread.
pub fn push_table(source: &str, payload: Payload) {
    state::cache_payload(source, &payload);
    let mut g = TABLES.lock().unwrap();
    if let Some(list) = g.as_mut().and_then(|m| m.get_mut(source)) {
        list.retain(|t| {
            let p = payload.clone();
            t.queue(move |m| m.apply_payload(p)).is_ok()
        });
    }
}

#[derive(Default)]
pub struct GenericTableRust {
    ncols: i32,
    src_keys: Vec<String>,
    src_rows: Vec<Row>,
    keys: Vec<String>,
    rows: Vec<Row>,
    sort_col: Option<usize>,
    sort_desc: bool,
    filter: String,
}

impl GenericTableRust {
    /// Apply sort and filter to the source rows.
    fn build_view(&self) -> (Vec<String>, Vec<Row>) {
        let needle = self.filter.to_lowercase();
        let mut idx: Vec<usize> = (0..self.src_rows.len())
            .filter(|&i| {
                needle.is_empty()
                    || self.src_rows[i]
                        .iter()
                        .any(|c| to_text(c).to_lowercase().contains(&needle))
            })
            .collect();
        if let Some(c) = self.sort_col {
            idx.sort_by(|&a, &b| {
                let o = cmp_cells(self.src_rows[a].get(c), self.src_rows[b].get(c));
                if self.sort_desc { o.reverse() } else { o }
            });
        }
        (
            idx.iter().map(|&i| self.src_keys[i].clone()).collect(),
            idx.iter().map(|&i| self.src_rows[i].clone()).collect(),
        )
    }
}

fn cmp_cells(a: Option<&Value>, b: Option<&Value>) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    match (a, b) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => {
            x.as_f64().partial_cmp(&y.as_f64()).unwrap_or(Equal)
        }
        (Some(x), Some(y)) => to_text(x).to_lowercase().cmp(&to_text(y).to_lowercase()),
        (None, Some(_)) => Less,
        (Some(_), None) => Greater,
        _ => Equal,
    }
}

impl qobject::GenericTable {
    // A flat table has no children: any valid parent index has zero rows and columns.
    fn row_count(&self, p: &QModelIndex) -> i32 {
        if p.is_valid() {
            0
        } else {
            self.rows.len() as i32
        }
    }
    fn column_count(&self, p: &QModelIndex) -> i32 {
        if p.is_valid() { 0 } else { self.ncols }
    }
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let (r, c) = (index.row(), index.column());
        let Some(cell) = (r >= 0 && c >= 0)
            .then(|| self.rows.get(r as usize).and_then(|x| x.get(c as usize)))
            .flatten()
        else {
            return QVariant::default();
        };
        match role {
            ROLE_DISPLAY => QVariant::from(&QString::from(to_text(cell).as_str())),
            ROLE_VALUE => to_variant(cell),
            ROLE_KEY => QVariant::from(&QString::from(self.keys[r as usize].as_str())),
            _ => QVariant::default(),
        }
    }
    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        role_names()
    }
    fn configure(mut self: Pin<&mut Self>, source: &QString, columns: i32) {
        let source = source.to_string();
        self.as_mut().rust_mut().get_mut().ncols = columns;
        let thread = self.qt_thread();
        {
            let mut g = TABLES.lock().unwrap();
            g.get_or_insert_with(HashMap::new)
                .entry(source.clone())
                .or_default()
                .push(thread);
        }
        if let Some(p) = state::cached_payload(&source) {
            self.apply_payload(Payload { append: false, ..p });
        }
    }
    fn count(&self) -> i32 {
        self.rows.len() as i32
    }
    fn key_at(&self, row: i32) -> QString {
        QString::from(self.keys.get(row as usize).map_or("", String::as_str))
    }
    fn row_of_key(&self, key: &QString) -> i32 {
        let k = key.to_string();
        self.keys
            .iter()
            .position(|x| *x == k)
            .map_or(-1, |i| i as i32)
    }
    fn cell(&self, row: i32, col: i32) -> QVariant {
        self.rows
            .get(row as usize)
            .and_then(|r| r.get(col as usize))
            .map_or_else(QVariant::default, to_variant)
    }
    fn sort_by(mut self: Pin<&mut Self>, col: i32, descending: bool) {
        {
            let st = self.as_mut().rust_mut().get_mut();
            st.sort_col = (col >= 0).then_some(col as usize);
            st.sort_desc = descending;
        }
        self.refresh_view();
    }
    fn set_filter(mut self: Pin<&mut Self>, text: &QString) {
        self.as_mut().rust_mut().get_mut().filter = text.to_string();
        self.refresh_view();
    }

    fn apply_payload(mut self: Pin<&mut Self>, p: Payload) {
        {
            let st = self.as_mut().rust_mut().get_mut();
            if p.append {
                st.src_keys.extend(p.keys);
                st.src_rows.extend(p.rows);
                if st.ncols == 0 {
                    st.ncols = st.src_rows.first().map_or(0, |r| r.len() as i32);
                }
            } else {
                st.src_keys = p.keys;
                st.src_rows = p.rows;
            }
        }
        self.refresh_view();
    }

    /// Recompute the view and emit the smallest sufficient model signals.
    fn refresh_view(mut self: Pin<&mut Self>) {
        enum Act {
            Same(Vec<usize>),
            Append(usize),
            Reset,
        }
        let (new_keys, new_rows) = self.build_view();
        let act = {
            let st = self.as_ref();
            if st.keys == new_keys {
                Act::Same(
                    (0..new_rows.len())
                        .filter(|&i| st.rows[i] != new_rows[i])
                        .collect(),
                )
            } else if new_keys.len() > st.keys.len() && new_keys[..st.keys.len()] == st.keys[..] {
                Act::Append(st.keys.len())
            } else {
                Act::Reset
            }
        };
        let cols = self.ncols;
        let n = new_rows.len();
        let store = move |mut m: Pin<&mut Self>| {
            let st = m.as_mut().rust_mut().get_mut();
            st.keys = new_keys;
            st.rows = new_rows;
        };
        match act {
            Act::Same(changed) => {
                store(self.as_mut());
                for i in changed {
                    self.as_mut().emit_rows_changed(i as i32, i as i32, cols);
                }
            }
            Act::Append(from) => self.mutate_insert(from as i32, n as i32 - 1, store),
            Act::Reset => self.mutate_reset(store),
        }
    }
}

// ========================================================================= TransferListModel

type TransferThread = CxxQtThread<qobject::TransferListModel>;
static TRANSFER: Mutex<Vec<TransferThread>> = Mutex::new(Vec::new());

/// Tell every transfer model that the shared store was updated.
pub fn notify_main(changes: MainChanges) {
    let mut g = TRANSFER.lock().unwrap();
    g.retain(|t| {
        let c = changes.clone();
        t.queue(move |m| m.on_main(&c)).is_ok()
    });
}

pub struct TransferListModelRust {
    view: ViewIndex,
    /// Row order as the model currently exposes it (advanced op by op).
    shown: Vec<String>,
    sort_col: i32,
    sort_desc: bool,
    /// Rows currently visible in the view (`None` until it reports them: then everything is signalled).
    visible: Option<(i32, i32)>,
}

impl Default for TransferListModelRust {
    fn default() -> Self {
        let sort_col = columns::index_of("name").unwrap_or(0) as i32;
        Self {
            view: ViewIndex::new(ViewQuery {
                sort_by: "name".into(),
                ..Default::default()
            }),
            shown: Vec::new(),
            sort_col,
            sort_desc: false,
            visible: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::TransferListModel {
    fn initialize(self: Pin<&mut Self>) {
        TRANSFER.lock().unwrap().push(self.qt_thread());
    }
}

impl qobject::TransferListModel {
    fn row_count(&self, p: &QModelIndex) -> i32 {
        if p.is_valid() {
            0
        } else {
            self.shown.len() as i32
        }
    }
    fn column_count(&self, p: &QModelIndex) -> i32 {
        if p.is_valid() {
            0
        } else {
            columns::COLUMNS.len() as i32
        }
    }
    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let (r, c) = (index.row(), index.column());
        if r < 0 || c < 0 {
            return QVariant::default();
        }
        let (Some(key), Some(col)) = (self.shown.get(r as usize), columns::COLUMNS.get(c as usize))
        else {
            return QVariant::default();
        };
        if role == ROLE_KEY {
            return QVariant::from(&QString::from(key.as_str()));
        }
        state::APP.with(|a| {
            let a = a.borrow();
            let Some(t) = a.store.torrents.get(key) else {
                return QVariant::default();
            };
            match role {
                ROLE_DISPLAY => QVariant::from(&QString::from(columns::display(col, t).as_str())),
                ROLE_VALUE => to_variant(&columns::raw(col, t)),
                _ => QVariant::default(),
            }
        })
    }
    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        role_names()
    }
    fn header(&self, col: i32) -> QString {
        QString::from(columns::COLUMNS.get(col as usize).map_or("", |c| c.title))
    }
    fn column_key(&self, col: i32) -> QString {
        QString::from(columns::COLUMNS.get(col as usize).map_or("", |c| c.key))
    }
    fn column_default_visible(&self, col: i32) -> bool {
        columns::COLUMNS
            .get(col as usize)
            .is_some_and(|c| c.default_visible)
    }
    fn hash_at(&self, row: i32) -> QString {
        QString::from(self.shown.get(row as usize).map_or("", String::as_str))
    }
    fn row_of_hash(&self, hash: &QString) -> i32 {
        let h = hash.to_string();
        self.shown
            .iter()
            .position(|x| *x == h)
            .map_or(-1, |i| i as i32)
    }
    fn text_at(&self, row: i32, key: &QString) -> QString {
        let key = key.to_string();
        let Some(h) = self.shown.get(row as usize) else {
            return QString::default();
        };
        let text = state::APP.with(|a| {
            let a = a.borrow();
            let t = a.store.torrents.get(h)?;
            Some(match columns::index_of(&key) {
                Some(i) => columns::display(&columns::COLUMNS[i], t),
                None => t.get(&key).map(to_text).unwrap_or_default(),
            })
        });
        QString::from(text.unwrap_or_default().as_str())
    }
    fn raw_at(&self, row: i32, key: &QString) -> QString {
        let key = key.to_string();
        let Some(h) = self.shown.get(row as usize) else {
            return QString::default();
        };
        let text = state::APP.with(|a| {
            a.borrow()
                .store
                .torrents
                .get(h)
                .and_then(|t| t.get(&key))
                .map(to_text)
        });
        QString::from(text.unwrap_or_default().as_str())
    }
    fn set_visible_rows(mut self: Pin<&mut Self>, first: i32, last: i32) {
        self.as_mut().rust_mut().get_mut().visible = Some((first, last));
    }
    fn sort_column(&self) -> i32 {
        self.sort_col
    }
    fn sort_descending(&self) -> bool {
        self.sort_desc
    }
    fn set_filter(mut self: Pin<&mut Self>, kind: &QString, key: &QString) {
        let (kind, key) = (kind.to_string(), key.to_string());
        {
            let q = &mut self.as_mut().rust_mut().get_mut().view.query;
            match kind.as_str() {
                "status" => {
                    q.status = StatusFilter::ALL
                        .into_iter()
                        .find(|f| f.api_name() == key)
                        .unwrap_or_default()
                }
                "category" => {
                    q.category = match key.as_str() {
                        "" => CategoryFilter::All,
                        "\u{0}uncategorized" => CategoryFilter::Uncategorized,
                        n => CategoryFilter::Named(n.to_owned()),
                    }
                }
                "tag" => {
                    q.tag = match key.as_str() {
                        "" => TagFilter::All,
                        "\u{0}untagged" => TagFilter::Untagged,
                        n => TagFilter::Named(n.to_owned()),
                    }
                }
                "tracker" => {
                    q.tracker = match key.as_str() {
                        "" => TrackerFilter::All,
                        "\u{0}trackerless" => TrackerFilter::Trackerless,
                        "\u{0}error" => TrackerFilter::Error,
                        "\u{0}warning" => TrackerFilter::Warning,
                        "\u{0}other" => TrackerFilter::OtherError,
                        h => TrackerFilter::Host(h.to_owned()),
                    }
                }
                _ => {}
            }
        }
        self.refresh();
    }
    fn set_search(mut self: Pin<&mut Self>, text: &QString, regex: bool) {
        {
            let q = &mut self.as_mut().rust_mut().get_mut().view.query;
            q.text = text.to_string();
            q.regex = regex;
        }
        self.refresh();
    }
    fn sort_by(mut self: Pin<&mut Self>, col: i32, descending: bool) {
        if let Some(c) = columns::COLUMNS.get(col as usize) {
            let st = self.as_mut().rust_mut().get_mut();
            st.sort_col = col;
            st.sort_desc = descending;
            st.view.query.sort_by = c.key.to_owned();
            st.view.query.reverse = descending;
        }
        self.refresh();
    }

    fn on_main(self: Pin<&mut Self>, changes: &MainChanges) {
        if std::env::var_os("QBREMOTE_PERF").is_none() {
            return self.on_main_inner(changes);
        }
        let t = std::time::Instant::now();
        self.on_main_inner(changes);
        eprintln!(
            "perf: model on_main {} ms ({} changed rows)",
            t.elapsed().as_millis(),
            changes.torrents.changed.len()
        );
    }

    fn on_main_inner(mut self: Pin<&mut Self>, changes: &MainChanges) {
        let perf = std::env::var_os("QBREMOTE_PERF").is_some();
        let t0 = std::time::Instant::now();
        // Only recompute the order when a changed field can move or hide rows.
        let affected = self.view.affected_by(changes);
        if affected {
            self.as_mut().refresh();
        }
        let t1 = t0.elapsed();
        if changes.torrents.changed.is_empty() {
            return;
        }
        let index: std::collections::HashMap<&str, usize> = self
            .shown
            .iter()
            .enumerate()
            .map(|(i, k)| (k.as_str(), i))
            .collect();
        let mut rows: Vec<usize> = changes
            .torrents
            .changed
            .iter()
            .filter_map(|(k, _)| index.get(k.as_str()).copied())
            .collect();
        rows.sort_unstable();
        // Off-screen cells are re-read when scrolled into view, so only signal rows near the viewport.
        if let Some((first, last)) = self.visible {
            const MARGIN: i32 = 40;
            let (lo, hi) = (
                (first - MARGIN).max(0) as usize,
                (last + MARGIN).max(0) as usize,
            );
            rows.retain(|r| (lo..=hi).contains(r));
        }
        if rows.is_empty() {
            return;
        }
        let t2 = t0.elapsed();
        // Repaint only the columns whose source fields changed (a speed tick touches ~4 of 39).
        let fields = changes
            .torrents
            .changed
            .iter()
            .flat_map(|(_, f)| f.iter().map(String::as_str));
        let dirty = columns::columns_affected_by(fields);
        // Contiguous column runs, so one signal covers each run.
        let mut runs: Vec<(i32, i32)> = Vec::new();
        for c in dirty {
            match runs.last_mut() {
                Some((_, last)) if *last + 1 == c as i32 => *last = c as i32,
                _ => runs.push((c as i32, c as i32)),
            }
        }
        // Many scattered changes: one range signal per column run beats thousands of per-row ones.
        if rows.len() > 64 {
            let (first, last) = (rows[0] as i32, *rows.last().unwrap() as i32);
            for (c0, c1) in runs {
                self.as_mut().emit_cells_changed(first, last, c0, c1);
            }
        } else {
            for r in rows {
                for &(c0, c1) in &runs {
                    self.as_mut().emit_cells_changed(r as i32, r as i32, c0, c1);
                }
            }
        }
        if perf {
            eprintln!(
                "perf:   refresh(affected={affected}) {} ms, row lookup {} ms, dataChanged emit {} ms",
                t1.as_millis(),
                (t2 - t1).as_millis(),
                (t0.elapsed() - t2).as_millis()
            );
        }
    }

    /// Recompute the view from the shared store and apply the resulting row operations.
    fn refresh(mut self: Pin<&mut Self>) {
        let ops = state::APP.with(|a| {
            let a = a.borrow();
            self.as_mut().rust_mut().get_mut().view.refresh(&a.store)
        });
        for op in ops {
            match op {
                RowOp::Remove { row } => {
                    // SAFETY: paired end call below; storage edited between them.
                    unsafe {
                        self.as_mut()
                            .begin_remove_rows(&no_parent(), row as i32, row as i32)
                    };
                    self.as_mut().rust_mut().get_mut().shown.remove(row);
                    unsafe { self.as_mut().end_remove_rows() };
                }
                RowOp::Insert { row, key } => {
                    self.as_mut()
                        .mutate_insert(row as i32, row as i32, move |mut m| {
                            m.as_mut().rust_mut().get_mut().shown.insert(row, key);
                        });
                }
                RowOp::Move { from, to } => {
                    // Qt's destination is "insert before this row" in pre-move numbering.
                    let dest = if to > from { to + 1 } else { to } as i32;
                    // SAFETY: paired with `end_move_rows` when begin succeeds.
                    let ok = unsafe {
                        self.as_mut().begin_move_rows(
                            &no_parent(),
                            from as i32,
                            from as i32,
                            &no_parent(),
                            dest,
                        )
                    };
                    let st = self.as_mut().rust_mut().get_mut();
                    let item = st.shown.remove(from);
                    st.shown.insert(to, item);
                    if ok {
                        unsafe { self.as_mut().end_move_rows() };
                    }
                }
                RowOp::Reset => {
                    self.as_mut().mutate_reset(|mut m| {
                        let st = m.as_mut().rust_mut().get_mut();
                        st.shown = st.view.rows().to_vec();
                    });
                }
            }
        }
    }
}

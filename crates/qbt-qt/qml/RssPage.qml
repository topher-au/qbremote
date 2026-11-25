import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

Item {
    id: root
    property bool active: false
    property string selectedPath: ""
    signal addRequested(string url)
    signal downloaderRequested()
    onActiveChanged: { if (active) AppController.watch("rss", ""); else AppController.unwatch("rss") }

    GenericTable { id: tree; Component.onCompleted: tree.configure("rss_tree", 6) }
    GenericTable { id: articles; Component.onCompleted: articles.configure("rss_articles", 8) }
    function refresh() { AppController.call("rssItems", "rss", "items", JSON.stringify({ withData: true })) }
    function done(tag) { Qt.callLater(refresh) }
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag.indexOf("rssop") === 0) { if (!ok) AppController.toast(body); root.refresh() }
        }
    }
    function op(action, params) { AppController.call("rssop" + action, "rss", action, JSON.stringify(params)) }
    function folderOf(path) { return path.indexOf("\\") >= 0 ? path.slice(0, path.lastIndexOf("\\")) : "" }
    function strip(html) { return String(html || "").replace(/<[^>]*>/g, " ").replace(/&nbsp;/g, " ").replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/\s+/g, " ").trim() }
    function selectedKind() { var r = treeTable.selectedRows(); return r.length ? tree.cell(r[0], 2) : "" }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        ToolBar {
            Layout.fillWidth: true
            Flow {
                anchors.fill: parent
                spacing: 4
                ToolButton { text: qsTr("New subscription…"); onClicked: addFeed.open() }
                ToolButton { text: qsTr("New folder…"); onClicked: addFolder.open() }
                ToolButton { text: qsTr("Update all"); onClicked: root.op("refreshItem", { itemPath: "" }) }
                ToolButton { text: qsTr("Update selected"); enabled: root.selectedPath.length > 0; onClicked: root.op("refreshItem", { itemPath: root.selectedPath }) }
                ToolButton { text: qsTr("Mark as read"); enabled: root.selectedPath.length > 0; onClicked: root.op("markAsRead", { itemPath: root.selectedPath }) }
                ToolButton { text: qsTr("Rename / move…"); enabled: root.selectedPath.length > 0; onClicked: movePrompt.open() }
                ToolButton { text: qsTr("Edit URL…"); enabled: root.selectedKind() === "feed"; onClicked: urlPrompt.open() }
                ToolButton { text: qsTr("Refresh interval…"); enabled: root.selectedKind() === "feed"; onClicked: intervalPrompt.open() }
                ToolButton { text: qsTr("Delete"); enabled: root.selectedPath.length > 0; onClicked: root.op("removeItem", { path: root.selectedPath }) }
                ToolButton { text: qsTr("Downloader rules…"); onClicked: root.downloaderRequested() }
            }
        }
        SplitView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            AppTable {
                id: treeTable
                SplitView.preferredWidth: 260
                table: tree
                sortable: false
                headers: [qsTr("Feeds"), ""]
                widths: [240, 0]
                Connections {
                    target: treeTable.view.selectionModel
                    function onSelectionChanged() {
                        var r = treeTable.selectedRows()
                        root.selectedPath = r.length ? tree.cell(r[0], 0) : ""
                        AppController.rssSelect(root.selectedPath)
                    }
                }
                cellDelegate: Rectangle {
            clip: true
                    required property bool selected
                    required property int row
                    required property int column
                    required property string display
                    implicitHeight: treeTable.rowHeight
                    color: selected ? treeTable.palette.highlight : treeTable.palette.base
                    Label {
                        anchors { fill: parent; leftMargin: 6 + tree.cell(row, 5) * 16 }
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                        text: (tree.cell(row, 2) === "folder" ? "📁 " : "📰 ") + tree.cell(row, 1) + (tree.cell(row, 3) > 0 ? " (" + tree.cell(row, 3) + ")" : "")
                        font.bold: tree.cell(row, 3) > 0
                        color: selected ? treeTable.palette.highlightedText : treeTable.palette.text
                    }
                }
            }
            SplitView {
                orientation: Qt.Vertical
                SplitView.fillWidth: true
                AppTable {
                    id: artTable
                    SplitView.preferredHeight: 300
                    table: articles
                    sortable: false
                    headers: [qsTr("Title"), qsTr("Date"), "", "", "", "", "", ""]
                    widths: [560, 180, 0, 0, 0, 0, 0, 0]
                    onActivated: function (row) { root.addRequested(articles.cell(row, 7)) }
                    onContextMenuRequested: function (row) { if (!isRowSelected(row)) selectRow(row); artMenu.popup() }
                    cellDelegate: Rectangle {
            clip: true
                        required property bool selected
                        required property int row
                        required property int column
                        required property string display
                        implicitHeight: artTable.rowHeight
                        color: selected ? artTable.palette.highlight : (row % 2 ? artTable.palette.alternateBase : artTable.palette.base)
                        Label {
                            anchors { fill: parent; leftMargin: 6; rightMargin: 6 }
                            text: display
                            elide: Text.ElideRight
                            verticalAlignment: Text.AlignVCenter
                            font.bold: articles.cell(row, 2) !== true
                            color: selected ? artTable.palette.highlightedText : artTable.palette.text
                        }
                        TapHandler { acceptedButtons: Qt.RightButton; onTapped: artTable.contextMenuRequested(row) }
                        TapHandler { acceptedButtons: Qt.LeftButton; onDoubleTapped: artTable.activated(row) }
                    }
                    Menu {
                        id: artMenu
                        MenuItem { text: qsTr("Download…"); onTriggered: { var r = artTable.selectedRows(); if (r.length) root.addRequested(articles.cell(r[0], 7)) } }
                        MenuItem { text: qsTr("Open news page"); onTriggered: { var r = artTable.selectedRows(); if (r.length) Qt.openUrlExternally(articles.cell(r[0], 4)) } }
                        MenuItem { text: qsTr("Mark as read"); onTriggered: { var r = artTable.selectedRows(); if (r.length) root.op("markAsRead", { itemPath: articles.cell(r[0], 6), articleId: articles.cell(r[0], 3) }) } }
                    }
                    Connections {
                        target: artTable.view.selectionModel
                        function onSelectionChanged() { var r = artTable.selectedRows(); desc.text = r.length ? root.strip(articles.cell(r[0], 5)) : "" }
                    }
                }
                ScrollView {
                    SplitView.fillHeight: true
                    TextArea { id: desc; readOnly: true; wrapMode: TextEdit.Wrap; placeholderText: qsTr("Select an article to see its description") }
                }
            }
        }
    }
    PromptDialog { id: addFeed; title: qsTr("New subscription"); prompt: qsTr("Feed URL:"); onSubmitted: function (t) { var f = root.selectedKind() === "folder" ? root.selectedPath : root.folderOf(root.selectedPath); root.op("addFeed", { url: t, path: (f.length ? f + "\\" : "") + t.replace(/^[a-z]+:\/\//, "").replace(/[\\\/:?*"<>|]/g, "_") }) } }
    PromptDialog { id: addFolder; title: qsTr("New folder"); prompt: qsTr("Folder name:"); onSubmitted: function (t) { var f = root.selectedKind() === "folder" ? root.selectedPath : root.folderOf(root.selectedPath); root.op("addFolder", { path: (f.length ? f + "\\" : "") + t }) } }
    PromptDialog { id: movePrompt; title: qsTr("Rename / move"); prompt: qsTr("New full path (use \\ between folders):"); text: root.selectedPath; onSubmitted: function (t) { root.op("moveItem", { itemPath: root.selectedPath, destPath: t }) } }
    PromptDialog { id: urlPrompt; title: qsTr("Feed URL"); prompt: qsTr("New URL:"); onSubmitted: function (t) { root.op("setFeedURL", { path: root.selectedPath, url: t }) } }
    PromptDialog { id: intervalPrompt; title: qsTr("Refresh interval"); prompt: qsTr("Seconds (0 = use default):"); numeric: true; text: "0"; onSubmitted: function (t) { root.op("setFeedRefreshInterval", { path: root.selectedPath, refreshInterval: parseInt(t) || 0 }) } }
}

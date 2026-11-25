import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

Item {
    id: root
    property bool active: false
    property int currentJob: -1
    signal addRequested(string url)
    signal pluginsRequested()

    GenericTable { id: jobs; Component.onCompleted: jobs.configure("search_jobs", 3) }
    GenericTable { id: results; Component.onCompleted: results.configure("search_results", 9) }

    onActiveChanged: {
        if (active) AppController.watch("searchstatus", "")
        else { AppController.unwatch("searchstatus"); AppController.unwatch("searchresults") }
    }
    function start() {
        if (pattern.text.trim().length === 0) return
        var plugins = pluginBox.currentIndex === 0 ? "enabled" : pluginBox.currentIndex === 1 ? "all" : pluginBox.currentText
        AppController.call("searchStart", "search", "start", JSON.stringify({ pattern: pattern.text, category: ["all", "movies", "tv", "music", "games", "anime", "software", "pictures", "books"][catBox.currentIndex], plugins: plugins }))
    }
    function show(id) { currentJob = id; AppController.watch("searchresults", String(id)) }
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag === "searchStart") { if (ok) root.show(JSON.parse(body).id); else AppController.toast(body) }
        }
    }
    GenericTable { id: pluginNames; Component.onCompleted: pluginNames.configure("plugins", 5) }

    ColumnLayout {
        anchors.fill: parent
        spacing: 4
        RowLayout {
            Layout.margins: 4
            TextField { id: pattern; Layout.fillWidth: true; placeholderText: qsTr("Search pattern"); onAccepted: root.start() }
            ComboBox { id: catBox; model: [qsTr("All categories"), qsTr("Movies"), qsTr("TV shows"), qsTr("Music"), qsTr("Games"), qsTr("Anime"), qsTr("Software"), qsTr("Pictures"), qsTr("Books")] }
            ComboBox { id: pluginBox; model: [qsTr("Enabled plugins"), qsTr("All plugins")] }
            Button { text: qsTr("Search"); highlighted: true; onClicked: root.start() }
            Button { text: qsTr("Stop"); enabled: root.currentJob >= 0; onClicked: AppController.call("searchStop", "search", "stop", JSON.stringify({ id: root.currentJob })) }
            Button { text: qsTr("Plugins…"); onClicked: root.pluginsRequested() }
        }
        TabBar {
            Layout.fillWidth: true
            visible: jobsRepeater.count > 0
            Repeater {
                id: jobsRepeater
                model: jobs
                TabButton {
                    required property int index
                    text: "#" + jobs.cell(index, 0) + " " + jobs.cell(index, 1) + " (" + jobs.cell(index, 2) + ")"
                    onClicked: root.show(jobs.cell(index, 0))
                    TapHandler {
                        acceptedButtons: Qt.MiddleButton | Qt.RightButton
                        onTapped: { AppController.call("searchDel", "search", "delete", JSON.stringify({ id: jobs.cell(index, 0) })); if (root.currentJob === jobs.cell(index, 0)) root.currentJob = -1 }
                    }
                }
            }
        }
        RowLayout {
            Layout.margins: 4
            TextField { id: filter; Layout.fillWidth: true; placeholderText: qsTr("Filter results"); onTextChanged: results.setFilter(text) }
            Label { text: qsTr("%1 results").arg(resTable.view.rows) }
        }
        AppTable {
            id: resTable
            Layout.fillWidth: true
            Layout.fillHeight: true
            table: results
            headers: [qsTr("Name"), qsTr("Size"), qsTr("Seeders"), qsTr("Leechers"), qsTr("Engine"), qsTr("Site"), "", "", qsTr("Published")]
            widths: [520, 100, 80, 80, 120, 200, 0, 0, 140]
            onHeaderSortRequested: function (c, d) { results.sortBy(c, d); sortColumn = c; sortDescending = d }
            onActivated: function (row) { root.addRequested(results.cell(row, 6)) }
            onContextMenuRequested: function (row) { if (!isRowSelected(row)) selectRow(row); menu.popup() }
            Menu {
                id: menu
                MenuItem { text: qsTr("Download…"); onTriggered: { var r = resTable.selectedRows(); if (r.length) root.addRequested(results.cell(r[0], 6)) } }
                MenuItem { text: qsTr("Download on server now"); onTriggered: { var r = resTable.selectedRows(); if (r.length) AppController.call("searchDl", "search", "downloadTorrent", JSON.stringify({ torrentUrl: results.cell(r[0], 6), pluginName: results.cell(r[0], 4) })) } }
                MenuItem { text: qsTr("Open description page"); onTriggered: { var r = resTable.selectedRows(); if (r.length) Qt.openUrlExternally(results.cell(r[0], 7)) } }
            }
        }
    }
}

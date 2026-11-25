import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Bottom panel for the selected torrent. Pollers run only while a tab is visible.
Item {
    id: root
    Accessible.role: Accessible.Pane
    Accessible.name: qsTr("Torrent properties")
    property string hash: ""            // single selected torrent, or ""
    property string hashes: ""          // all selected, pipe-separated
    property bool active: true          // panel is shown
    signal dialogRequested(string name, var args)
    signal action(string name, string hashes, string arg)

    function prioText(p) { return p === -1 ? qsTr("Mixed") : p === 0 ? qsTr("Do not download") : p === 1 ? qsTr("Normal") : p === 6 ? qsTr("High") : p === 7 ? qsTr("Maximum") : String(p) }
    function showTab(i) { bar.currentIndex = i }
    function refreshWatch() {
        var kinds = ["general", "trackers", "peers", "webseeds", "content"]
        for (var i = 0; i < kinds.length; ++i) AppController.unwatch(kinds[i])
        if (!root.active || root.hash === "") return
        var k = kinds[bar.currentIndex]
        if (bar.currentIndex < 5) AppController.watch(k, root.hash)
    }
    onHashChanged: refreshWatch()
    onActiveChanged: refreshWatch()

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        TabBar {
            id: bar
            Layout.fillWidth: true
            onCurrentIndexChanged: root.refreshWatch()
            TabButton { text: qsTr("General") }
            TabButton { text: qsTr("Trackers") }
            TabButton { text: qsTr("Peers") }
            TabButton { text: qsTr("HTTP Sources") }
            TabButton { text: qsTr("Content") }
            TabButton { text: qsTr("Speed") }
        }
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: bar.currentIndex

            // ---------------- General
            Item {
                ColumnLayout {
                    anchors { fill: parent; margins: 6 }
                    RowLayout {
                        Label { text: qsTr("Pieces"); font.bold: true }
                        Item { Layout.fillWidth: true }
                        Button { text: qsTr("Copy piece hashes"); enabled: root.hash !== ""; onClicked: AppController.call("pieceHashes", "torrents", "pieceHashes", JSON.stringify({ hash: root.hash })) }
                    }
                    PieceBar { Layout.fillWidth: true; states: AppController.pieceStates }
                    Label { text: qsTr("Availability"); font.bold: true }
                    PieceBar { Layout.fillWidth: true; availability: true; values: AppController.pieceAvailability }
                    GenericTable { id: general; Component.onCompleted: general.configure("general", 2) }
                    AppTable {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        table: general
                        headers: [qsTr("Property"), qsTr("Value")]
                        widths: [180, 900]
                        sortable: false
                    }
                }
                Label { anchors.centerIn: parent; visible: root.hash === ""; text: qsTr("Select a torrent"); opacity: 0.6 }
            }

            // ---------------- Trackers
            Item {
                GenericTable { id: trk; Component.onCompleted: trk.configure("trackers", 10) }
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0
                    RowLayout {
                        Layout.margins: 4
                        Button { text: qsTr("Add…"); enabled: root.hash !== ""; onClicked: root.dialogRequested("addTrackers", {}) }
                        Button { text: qsTr("Edit…"); enabled: trkTable.selectedRows().length === 1 && trk.cell(trkTable.selectedRows()[0], 9) === 0; onClicked: root.dialogRequested("editTracker", { url: trk.cell(trkTable.selectedRows()[0], 0), tier: trk.cell(trkTable.selectedRows()[0], 2) }) }
                        Button { text: qsTr("Remove"); enabled: trkTable.selectedRows().length > 0; onClicked: root.action("removeTrackers", root.hash, trkUrls().join("\n")) }
                        Button { text: qsTr("Reannounce"); enabled: root.hash !== ""; onClicked: root.action("reannounce", root.hash, "") }
                        Button { text: qsTr("Copy URL"); enabled: trkTable.selectedRows().length > 0; onClicked: root.dialogRequested("copyText", { text: trkUrls().join("\n") }) }
                    }
                    function trkUrls() {
                        var rows = trkTable.selectedRows(), out = []
                        for (var i = 0; i < rows.length; ++i) if (trk.cell(rows[i], 9) === 0) out.push(trk.cell(rows[i], 0))
                        return out
                    }
                    AppTable {
                        id: trkTable
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        table: trk
                        sortable: false
                        headers: [qsTr("URL"), qsTr("Status"), qsTr("Tier"), qsTr("Peers"), qsTr("Seeds"), qsTr("Leeches"), qsTr("Downloaded"), qsTr("Message"), qsTr("Next announce"), ""]
                        widths: [380, 130, 50, 60, 60, 70, 90, 260, 110, 0]
                        cellDelegate: Rectangle {
            clip: true
                            required property bool selected
                            required property int row
                            required property int column
                            required property string display
                            implicitHeight: trkTable.rowHeight
                            color: selected ? trkTable.palette.highlight : (row % 2 ? trkTable.palette.alternateBase : trkTable.palette.base)
                            Label {
                                anchors { fill: parent; leftMargin: 6 + (column === 0 ? trk.cell(row, 9) * 18 : 0); rightMargin: 6 }
                                text: display
                                elide: Text.ElideRight
                                verticalAlignment: Text.AlignVCenter
                                color: selected ? trkTable.palette.highlightedText : trkTable.palette.text
                                opacity: trk.cell(row, 9) === 1 ? 0.75 : 1
                            }
                        }
                    }
                }
            }

            // ---------------- Peers
            Item {
                GenericTable { id: peers; Component.onCompleted: peers.configure("peers", 13) }
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0
                    RowLayout {
                        Layout.margins: 4
                        Button { text: qsTr("Add peers…"); enabled: root.hash !== ""; onClicked: root.dialogRequested("addPeers", {}) }
                        Button { text: qsTr("Ban selected"); enabled: peerTable.selectedRows().length > 0; onClicked: root.dialogRequested("banPeers", { peers: peerAddrs() }) }
                        Button { text: qsTr("Copy address"); enabled: peerTable.selectedRows().length > 0; onClicked: root.dialogRequested("copyText", { text: peerAddrs().join("\n") }) }
                    }
                    function peerAddrs() {
                        var rows = peerTable.selectedRows(), out = []
                        for (var i = 0; i < rows.length; ++i) out.push(peers.cell(rows[i], 0))
                        return out
                    }
                    AppTable {
                        id: peerTable
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        table: peers
                        sortable: true
                        sortColumn: -1
                        onHeaderSortRequested: function (c, d) { peers.sortBy(c, d); sortColumn = c; sortDescending = d }
                        headers: [qsTr("IP:port"), qsTr("Client"), qsTr("Connection"), qsTr("Flags"), qsTr("Country"), qsTr("Progress"), qsTr("Down"), qsTr("Up"), qsTr("Downloaded"), qsTr("Uploaded"), qsTr("Relevance"), qsTr("Files"), qsTr("Host")]
                        widths: [180, 150, 90, 70, 140, 80, 90, 90, 90, 90, 80, 200, 200]
                    }
                }
            }

            // ---------------- HTTP sources
            Item {
                GenericTable { id: ws; Component.onCompleted: ws.configure("webseeds", 1) }
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0
                    RowLayout {
                        Layout.margins: 4
                        Button { text: qsTr("Add…"); enabled: root.hash !== ""; onClicked: root.dialogRequested("addWebSeeds", {}) }
                        Button { text: qsTr("Edit…"); enabled: wsTable.selectedRows().length === 1; onClicked: root.dialogRequested("editWebSeed", { url: ws.cell(wsTable.selectedRows()[0], 0) }) }
                        Button {
                            text: qsTr("Remove")
                            enabled: wsTable.selectedRows().length > 0
                            onClicked: { var r = wsTable.selectedRows(), u = []; for (var i = 0; i < r.length; ++i) u.push(ws.cell(r[i], 0)); root.action("removeWebSeeds", root.hash, u.join("\n")) }
                        }
                    }
                    AppTable { id: wsTable; Layout.fillWidth: true; Layout.fillHeight: true; table: ws; sortable: false; headers: [qsTr("URL")]; widths: [900] }
                }
            }

            // ---------------- Content
            Item {
                GenericTable { id: files; Component.onCompleted: files.configure("files", 10) }
                // (defined on the panel root below)
                function _unused() {}
                function prioTextLocal(p) { return p === -1 ? qsTr("Mixed") : p === 0 ? qsTr("Do not download") : p === 1 ? qsTr("Normal") : p === 6 ? qsTr("High") : p === 7 ? qsTr("Maximum") : String(p) }
                // File indexes under the selected rows (a folder covers all files below it).
                function selectedIndexes() {
                    var rows = fileTable.selectedRows(), set = {}
                    for (var i = 0; i < rows.length; ++i) {
                        if (!files.cell(rows[i], 8)) { set[files.cell(rows[i], 5)] = true; continue }
                        var pre = files.cell(rows[i], 6) + "/"
                        for (var r = 0; r < fileTable.view.rows; ++r) {
                            var idx = files.cell(r, 5)
                            if (idx >= 0 && String(files.cell(r, 6)).indexOf(pre) === 0) set[idx] = true
                        }
                    }
                    return Object.keys(set).map(Number)
                }
                function setPrio(p) { var ids = selectedIndexes(); if (ids.length) root.action("filePrio", root.hash, ids.join(",") + "\x1f" + p) }
                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0
                    AppTable {
                        id: fileTable
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        table: files
                        sortable: false
                        headers: [qsTr("Name"), qsTr("Size"), qsTr("Progress"), qsTr("Priority"), qsTr("Availability"), "", "", "", "", qsTr("Pieces")]
                        widths: [420, 90, 80, 130, 90, 0, 0, 0, 0, 120]
                        onContextMenuRequested: function (row) { if (!isRowSelected(row)) selectRow(row); fileMenu.popup() }
                        cellDelegate: Rectangle {
            clip: true
                            required property bool selected
                            required property int row
                            required property int column
                            required property string display
                            implicitHeight: fileTable.rowHeight
                            color: selected ? fileTable.palette.highlight : (row % 2 ? fileTable.palette.alternateBase : fileTable.palette.base)
                            Label {
                                anchors { fill: parent; leftMargin: 6 + (column === 0 ? files.cell(row, 7) * 18 : 0); rightMargin: 6 }
                                text: column === 3 ? root.prioText(files.cell(row, 3)) : (column === 0 && files.cell(row, 8) ? "▸ " + display : display)
                                elide: Text.ElideRight
                                verticalAlignment: Text.AlignVCenter
                                color: selected ? fileTable.palette.highlightedText : fileTable.palette.text
                                opacity: column === 3 && files.cell(row, 3) === 0 ? 0.6 : 1
                            }
                            TapHandler { acceptedButtons: Qt.RightButton; onTapped: fileTable.contextMenuRequested(row) }
                        }
                    }
                }
                Menu {
                    id: fileMenu
                    MenuItem { text: qsTr("Do not download"); onTriggered: setPrio(0) }
                    MenuItem { text: qsTr("Normal"); onTriggered: setPrio(1) }
                    MenuItem { text: qsTr("High"); onTriggered: setPrio(6) }
                    MenuItem { text: qsTr("Maximum"); onTriggered: setPrio(7) }
                    MenuSeparator {}
                    MenuItem { text: qsTr("Rename…"); enabled: fileTable.selectedRows().length === 1; onTriggered: { var r = fileTable.selectedRows()[0]; root.dialogRequested("renamePath", { path: files.cell(r, 6), folder: files.cell(r, 8) }) } }
                    MenuItem {
                        text: qsTr("Download to this computer…")
                        enabled: AppController.supports("torrents", "downloadFile") && fileTable.selectedRows().length === 1 && !files.cell(fileTable.selectedRows()[0], 8)
                        onTriggered: { var r = fileTable.selectedRows()[0]; root.dialogRequested("downloadFile", { path: files.cell(r, 6), name: files.cell(r, 0) }) }
                    }
                }
            }

            // ---------------- Speed
            SpeedGraph {}
        }
    }
}

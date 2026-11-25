import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.qbremote

// Create a .torrent on the server from a server-side path.
Dialog {
    id: root
    title: qsTr("Torrent creator")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(860, (parent ? parent.width : 860) - 40)
    height: Math.min(700, (parent ? parent.height : 700) - 40)
    standardButtons: Dialog.Close
    property string message: ""
    GenericTable { id: tasks; Component.onCompleted: tasks.configure("creator", 6) }
    onAboutToShow: AppController.watch("creator", "")
    onClosed: AppController.unwatch("creator")

    Connections {
        target: AppController
        function onCallResult(tag, ok, body) { if (tag === "creatorAdd") root.message = ok ? qsTr("Task created.") : body }
    }

    function create() {
        var p = { sourcePath: source.text, format: ["v1", "v2", "hybrid"][format.currentIndex],
            pieceSize: [0, 16384, 32768, 65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432][pieceSize.currentIndex],
            private: privateBox.checked, startSeeding: seedBox.checked, optimizeAlignment: alignBox.checked,
            ignoreDotfiles: dotBox.checked, comment: comment.text, source: srcField.text,
            trackers: trackers.text.split("\n").filter(function (s) { return s.trim().length }).join("\n"),
            urlSeeds: seeds.text.split("\n").filter(function (s) { return s.trim().length }).join("\n") }
        if (target.text.length) p.torrentFilePath = target.text
        if (padded.value >= 0) p.paddedFileSizeLimit = padded.value
        AppController.call("creatorAdd", "torrentcreator", "addTask", JSON.stringify(p))
    }

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            GridLayout {
                columns: 3
                Layout.fillWidth: true
                Label { text: qsTr("Source (server path)") }
                TextField { id: source; Layout.fillWidth: true }
                Button { text: qsTr("Browse…"); onClicked: { picker.path = source.text || "/"; picker.target = "src"; picker.open() } }
                Label { text: qsTr("Save .torrent on server (optional)") }
                TextField { id: target; Layout.fillWidth: true }
                Button { text: qsTr("Browse…"); onClicked: { picker.path = target.text || "/"; picker.target = "dst"; picker.open() } }
                Label { text: qsTr("Format") }
                ComboBox { id: format; Layout.columnSpan: 2; model: ["v1", "v2", qsTr("Hybrid")] ; currentIndex: 2 }
                Label { text: qsTr("Piece size") }
                ComboBox { id: pieceSize; Layout.columnSpan: 2; model: [qsTr("Auto"), "16 KiB", "32 KiB", "64 KiB", "128 KiB", "256 KiB", "512 KiB", "1 MiB", "2 MiB", "4 MiB", "8 MiB", "16 MiB", "32 MiB"] }
                Label { text: qsTr("Padded file size limit (bytes, -1 default)") }
                SpinBox { id: padded; Layout.columnSpan: 2; from: -1; to: 2147483647; value: -1; editable: true }
                Label { text: qsTr("Trackers (one per line)") }
                ScrollView { Layout.columnSpan: 2; Layout.fillWidth: true; Layout.preferredHeight: 60; TextArea { id: trackers } }
                Label { text: qsTr("Web seeds (one per line)") }
                ScrollView { Layout.columnSpan: 2; Layout.fillWidth: true; Layout.preferredHeight: 50; TextArea { id: seeds } }
                Label { text: qsTr("Comment") }
                TextField { id: comment; Layout.columnSpan: 2; Layout.fillWidth: true }
                Label { text: qsTr("Source tag") }
                TextField { id: srcField; Layout.columnSpan: 2; Layout.fillWidth: true }
            }
            RowLayout {
                CheckBox { id: privateBox; text: qsTr("Private torrent") }
                CheckBox { id: seedBox; text: qsTr("Start seeding immediately") }
                CheckBox { id: alignBox; text: qsTr("Optimize alignment"); checked: true }
                CheckBox { id: dotBox; text: qsTr("Ignore dotfiles") }
            }
            RowLayout {
                Button { text: qsTr("Create"); highlighted: true; enabled: source.text.length > 0; onClicked: root.create() }
                Label { text: root.message; Layout.fillWidth: true; elide: Text.ElideRight }
            }
            Label { text: qsTr("Tasks"); font.bold: true }
            AppTable {
                id: taskTable
                Layout.fillWidth: true
                Layout.preferredHeight: 150
                table: tasks
                sortable: false
                headers: [qsTr("Task"), qsTr("Source"), qsTr("Status"), qsTr("Progress"), qsTr("Error"), qsTr("Added")]
                widths: [150, 280, 80, 80, 200, 140]
            }
            RowLayout {
                Button {
                    text: qsTr("Download .torrent…")
                    enabled: taskTable.selectedRows().length === 1 && tasks.cell(taskTable.selectedRows()[0], 2) === "Finished"
                    onClicked: saveDialog.open()
                }
                Button {
                    text: qsTr("Delete task")
                    enabled: taskTable.selectedRows().length > 0
                    onClicked: { var r = taskTable.selectedRows(); for (var i = 0; i < r.length; ++i) AppController.call("creatorDel", "torrentcreator", "deleteTask", JSON.stringify({ taskID: tasks.cell(r[i], 0) })) }
                }
            }
        }
    }
    FileDialog {
        id: saveDialog
        fileMode: FileDialog.SaveFile
        nameFilters: ["Torrent (*.torrent)"]
        onAccepted: AppController.saveToFile("torrentcreator", "torrentFile", JSON.stringify({ taskID: tasks.cell(taskTable.selectedRows()[0], 0) }), AppController.localPath(selectedFile.toString()))
    }
    RemotePathPicker {
        id: picker
        property string target: "src"
        mode: "all"
        onPicked: function (p) { if (target === "src") source.text = p; else root.targetPath(p) }
    }
    function targetPath(p) { target.text = p }
}

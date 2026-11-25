import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.qbremote

// Add torrents from links/magnets and local .torrent files, with options and a file preview.
Dialog {
    id: root
    title: qsTr("Add torrents")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(820, (parent ? parent.width : 820) - 40)
    height: Math.min(700, (parent ? parent.height : 700) - 40)
    property string files: ""         // newline-separated local paths
    property string urls: ""
    property var skipped: ({})
    property int fileCount: 0
    signal submitted(string urls, string files, string optionsJson)

    function openWith(urlText, fileList) {
        urls = urlText || ""
        files = fileList || ""
        linksArea.text = urls
        skipped = ({})
        open()
    }
    GenericTable { id: preview; Component.onCompleted: preview.configure("preview", 10) }
    // Files the server excludes by its filter (priority 0) start unchecked.
    Connections {
        target: preview
        function onModelReset() {
            var s = {}
            for (var r = 0; r < preview.count(); ++r) if (!preview.cell(r, 8) && preview.cell(r, 3) === 0) s[preview.cell(r, 5)] = true
            root.skipped = s
        }
    }
    GenericTable { id: cats; Component.onCompleted: cats.configure("category_names", 1) }

    function sources() {
        var u = linksArea.text.split("\n").map(function (s) { return s.trim() }).filter(function (s) { return s.length })
        var f = root.files.split("\n").filter(function (s) { return s.trim().length })
        return { urls: u, files: f, count: u.length + f.length }
    }
    function loadPreview() {
        var s = sources()
        AppController.previewCancel(s.urls.length ? s.urls[0] : "")
        skipped = ({})
        if (s.count !== 1) return
        if (s.files.length === 1) AppController.previewFiles(s.files[0])
        else if (s.urls[0].indexOf("magnet:") === 0 || s.urls[0].indexOf("http") === 0) AppController.previewMagnet(s.urls[0])
    }
    function setSkip(row, skip) {
        var s = skipped
        if (preview.cell(row, 8)) {
            var pre = preview.cell(row, 6) + "/"
            for (var r = 0; r < preview.count(); ++r) {
                var idx = preview.cell(r, 5)
                if (idx >= 0 && String(preview.cell(r, 6)).indexOf(pre) === 0) s[idx] = skip
            }
        } else s[preview.cell(row, 5)] = skip
        skipped = s
        previewTable.view.forceLayout()
    }
    function options() {
        var o = {}
        if (savePath.text.length) o.savepath = savePath.text
        if (category.editText.length) o.category = category.editText
        if (tagsField.text.length) o.tags = tagsField.text
        if (renameField.text.length) o.rename = renameField.text
        o.contentLayout = ["Original", "Subfolder", "NoSubfolder"][layoutBox.currentIndex]
        o.stopCondition = ["None", "MetadataReceived", "FilesChecked"][stopBox.currentIndex]
        o.stopped = !startBox.checked
        o.addToTopOfQueue = topBox.checked
        o.seedMode = skipHash.checked
        o.sequentialDownload = seqBox.checked
        o.firstLastPiecePrio = flBox.checked
        o.autoTMM = tmmBox.checked
        if (dlLimit.value > 0) o.dlLimit = dlLimit.value * 1024
        if (ulLimit.value > 0) o.upLimit = ulLimit.value * 1024
        // Per-file priorities only for a single source with a loaded preview.
        var s = sources()
        if (s.count === 1 && preview.count() > 0) {
            var n = 0, pr = []
            for (var r = 0; r < preview.count(); ++r) if (!preview.cell(r, 8)) n++
            for (var i = 0; i < n; ++i) pr.push(skipped[i] ? 0 : 1)
            if (pr.indexOf(0) >= 0) o.filePriorities = pr.join(",")
        }
        return JSON.stringify(o)
    }

    Connections {
        target: AppController
        function onCallResult(tag, ok, body) { if (tag === "defSave" && ok && !savePath.text.length) savePath.text = body.trim() }
        function onPrefsLoaded() {
            if (!savePath.text.length) savePath.text = AppController.pref("save_path")
            startBox.checked = AppController.pref("add_stopped_enabled") !== "true"
            topBox.checked = AppController.pref("add_to_top_of_queue") === "true"
            var l = AppController.pref("torrent_content_layout")
            layoutBox.currentIndex = Math.max(0, ["Original", "Subfolder", "NoSubfolder"].indexOf(l))
            tmmBox.checked = AppController.pref("auto_tmm_enabled") === "true"
        }
    }
    onAboutToShow: { AppController.prefsLoad(); AppController.call("defSave", "app", "defaultSavePath", ""); loadPreview() }
    onClosed: { var s = sources(); if (s.urls.length) AppController.previewCancel(s.urls[0]) }
    onAccepted: { var s = sources(); root.submitted(s.urls.join("\n"), s.files.join("\n"), options()) }

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth
        ColumnLayout {
            width: parent.width
            spacing: 8
            GroupBox {
                title: qsTr("Torrents")
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    Label { text: qsTr("Links (one per line: magnet:, http(s)://, bc://bt/):") }
                    ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 70; TextArea { id: linksArea; wrapMode: TextEdit.NoWrap; onTextChanged: root.urls = text } }
                    RowLayout {
                        Button { text: qsTr("Add .torrent files…"); onClicked: fileDialog.open() }
                        Label { Layout.fillWidth: true; elide: Text.ElideMiddle; text: root.files.length ? root.files.split("\n").filter(function (s) { return s.length }).length + qsTr(" file(s) selected") : qsTr("No local files") }
                        Button { text: qsTr("Clear files"); enabled: root.files.length > 0; onClicked: root.files = "" }
                        Button { text: qsTr("Preview files"); onClicked: root.loadPreview() }
                        Button { text: qsTr("Save .torrent…"); enabled: preview.count() > 0 && root.sources().urls.length === 1; onClicked: saveMeta.open() }
                    }
                }
            }
            GroupBox {
                title: qsTr("Save options")
                Layout.fillWidth: true
                GridLayout {
                    anchors.fill: parent
                    columns: 4
                    Label { text: qsTr("Save at (server path)") }
                    RowLayout {
                        Layout.columnSpan: 3
                        Layout.fillWidth: true
                        TextField { id: savePath; Layout.fillWidth: true }
                        Button { text: qsTr("Browse…"); onClicked: { picker.path = savePath.text.length ? savePath.text : "/"; picker.open() } }
                    }
                    Label { text: qsTr("Category") }
                    ComboBox {
                        id: category
                        editable: true
                        Layout.fillWidth: true
                        model: cats
                        textRole: "display"
                        currentIndex: -1
                        editText: ""
                    }
                    Label { text: qsTr("Tags") }
                    TextField { id: tagsField; Layout.fillWidth: true; placeholderText: qsTr("comma-separated") }
                    Label { text: qsTr("Rename") }
                    TextField { id: renameField; Layout.fillWidth: true }
                    Label { text: qsTr("Content layout") }
                    ComboBox { id: layoutBox; Layout.fillWidth: true; model: [qsTr("Original"), qsTr("Create subfolder"), qsTr("Don't create subfolder")] }
                    Label { text: qsTr("Stop condition") }
                    ComboBox { id: stopBox; Layout.fillWidth: true; model: [qsTr("None"), qsTr("Metadata received"), qsTr("Files checked")] }
                    Label { text: qsTr("Limits (KiB/s)") }
                    RowLayout {
                        Layout.fillWidth: true
                        Label { text: "↓" }
                        SpinBox { id: dlLimit; from: 0; to: 10000000; editable: true }
                        Label { text: "↑" }
                        SpinBox { id: ulLimit; from: 0; to: 10000000; editable: true }
                    }
                }
            }
            GridLayout {
                columns: 3
                Layout.fillWidth: true
                CheckBox { id: tmmBox; text: qsTr("Automatic torrent management") }
                CheckBox { id: startBox; text: qsTr("Start torrent"); checked: true }
                CheckBox { id: topBox; text: qsTr("Add to top of queue") }
                CheckBox { id: skipHash; text: qsTr("Skip hash check") }
                CheckBox { id: seqBox; text: qsTr("Download in sequential order") }
                CheckBox { id: flBox; text: qsTr("Download first and last pieces first") }
            }
            GroupBox {
                title: qsTr("Files")
                Layout.fillWidth: true
                visible: preview.count() > 0
                ColumnLayout {
                    anchors.fill: parent
                    RowLayout {
                        Button { text: qsTr("Select all"); onClicked: { root.skipped = ({}); previewTable.view.forceLayout() } }
                        Button { text: qsTr("Select none"); onClicked: { var s = {}; for (var r = 0; r < preview.count(); ++r) if (!preview.cell(r, 8)) s[preview.cell(r, 5)] = true; root.skipped = s; previewTable.view.forceLayout() } }
                    }
                    AppTable {
                        id: previewTable
                        Layout.fillWidth: true
                        Layout.preferredHeight: 200
                        table: preview
                        sortable: false
                        headers: [qsTr("Name"), qsTr("Size"), "", qsTr("Download"), "", "", "", "", "", ""]
                        widths: [460, 100, 0, 90, 0, 0, 0, 0, 0, 0]
                        cellDelegate: Rectangle {
            clip: true
                            required property bool selected
                            required property int row
                            required property int column
                            required property string display
                            implicitHeight: previewTable.rowHeight
                            color: row % 2 ? previewTable.palette.alternateBase : previewTable.palette.base
                            Label {
                                visible: column !== 3
                                anchors { fill: parent; leftMargin: 6 + (column === 0 ? preview.cell(row, 7) * 18 : 0) }
                                text: display
                                elide: Text.ElideRight
                                verticalAlignment: Text.AlignVCenter
                            }
                            CheckBox {
                                visible: column === 3
                                anchors.centerIn: parent
                                checked: !(preview.cell(row, 8) ? false : root.skipped[preview.cell(row, 5)] === true)
                                onClicked: root.setSkip(row, !checked)
                            }
                        }
                    }
                }
            }
        }
    }

    FileDialog {
        id: fileDialog
        title: qsTr("Torrent files")
        fileMode: FileDialog.OpenFiles
        nameFilters: [qsTr("Torrent files (*.torrent)"), qsTr("All files (*)")]
        onAccepted: {
            var l = []
            for (var i = 0; i < selectedFiles.length; ++i) l.push(AppController.localPath(selectedFiles[i].toString()))
            root.files = (root.files.length ? root.files + "\n" : "") + l.join("\n")
            root.loadPreview()
        }
    }
    FileDialog {
        id: saveMeta
        fileMode: FileDialog.SaveFile
        nameFilters: ["Torrent (*.torrent)"]
        onAccepted: AppController.saveToFile("torrents", "saveMetadata", JSON.stringify({ source: root.sources().urls[0] }), AppController.localPath(selectedFile.toString()))
    }
    RemotePathPicker { id: picker; onPicked: function (p) { savePath.text = p } }
    standardButtons: Dialog.Ok | Dialog.Cancel
}

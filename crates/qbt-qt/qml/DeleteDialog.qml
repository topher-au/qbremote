import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: root
    property int count: 0
    property bool deleteFilesDefault: false
    property bool forceFiles: false     // Shift+Del: start with "delete files" ticked
    signal confirmed(bool deleteFiles)
    title: qsTr("Delete torrents")
    modal: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.Yes | Dialog.Cancel
    onAboutToShow: files.checked = deleteFilesDefault || forceFiles
    onClosed: forceFiles = false
    onAccepted: root.confirmed(files.checked)
    ColumnLayout {
        Label { text: root.count === 1 ? qsTr("Are you sure you want to delete the selected torrent from the transfer list?") : qsTr("Are you sure you want to delete the %1 selected torrents from the transfer list?").arg(root.count) }
        CheckBox { id: files; text: qsTr("Also permanently delete the files on the server"); Accessible.name: text }
    }
}

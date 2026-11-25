import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Create or edit a category: name, save path, download path.
Dialog {
    id: root
    property bool editing: false
    property string categoryName: ""
    property string savePath: ""
    signal submitted(bool editing, string name, string savePath, string downloadPath)
    title: editing ? qsTr("Edit category") : qsTr("New category")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(560, (parent ? parent.width : 560) - 40)
    standardButtons: Dialog.Ok | Dialog.Cancel
    onAboutToShow: { nameField.text = categoryName; pathField.text = savePath; dlPath.text = "" }
    onAccepted: root.submitted(root.editing, nameField.text, pathField.text, dlPath.text)
    GridLayout {
        anchors.fill: parent
        columns: 3
        Label { text: qsTr("Name") }
        TextField { id: nameField; Layout.fillWidth: true; Layout.columnSpan: 2; enabled: !root.editing }
        Label { text: qsTr("Save path") }
        TextField { id: pathField; Layout.fillWidth: true }
        Button { text: qsTr("Browse…"); onClicked: { picker.path = pathField.text || "/"; picker.target = "save"; picker.open() } }
        Label { text: qsTr("Incomplete path") }
        TextField { id: dlPath; Layout.fillWidth: true }
        Button { text: qsTr("Browse…"); onClicked: { picker.path = dlPath.text || "/"; picker.target = "dl"; picker.open() } }
    }
    RemotePathPicker {
        id: picker
        property string target: "save"
        onPicked: function (p) { if (target === "save") pathField.text = p; else dlPath.text = p }
    }
}

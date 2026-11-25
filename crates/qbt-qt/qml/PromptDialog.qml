import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Single-line (or multi-line) text prompt.
Dialog {
    id: root
    property string prompt: ""
    property string text: ""
    property bool multiline: false
    property bool numeric: false
    property string hint: ""
    signal submitted(string text)
    title: ""
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(560, (parent ? parent.width : 560) - 40)
    standardButtons: Dialog.Ok | Dialog.Cancel
    onAboutToShow: { field.text = root.text; area.text = root.text; (multiline ? area : field).forceActiveFocus(); (multiline ? area : field).selectAll() }
    onAccepted: root.submitted(multiline ? area.text : field.text)
    ColumnLayout {
        anchors.fill: parent
        Label { text: root.prompt; wrapMode: Text.Wrap; Layout.fillWidth: true; visible: text.length > 0 }
        TextField {
            id: field
            visible: !root.multiline
            Layout.fillWidth: true
            inputMethodHints: root.numeric ? Qt.ImhFormattedNumbersOnly : Qt.ImhNone
            onAccepted: root.accept()
        }
        ScrollView {
            visible: root.multiline
            Layout.fillWidth: true
            Layout.preferredHeight: 140
            TextArea { id: area; wrapMode: TextEdit.NoWrap }
        }
        Label { text: root.hint; visible: text.length > 0; opacity: 0.7; wrapMode: Text.Wrap; Layout.fillWidth: true }
    }
}

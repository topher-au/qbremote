import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Per-torrent share limits. Values: -2 use global, -1 unlimited.
Dialog {
    id: root
    title: qsTr("Share limits")
    modal: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.Ok | Dialog.Cancel
    signal submitted(string arg)
    onAccepted: root.submitted([
        ratioMode.currentIndex === 0 ? "-2" : ratioMode.currentIndex === 1 ? "-1" : ratio.value / 100,
        timeMode.currentIndex === 0 ? "-2" : timeMode.currentIndex === 1 ? "-1" : minutes.value,
        inactMode.currentIndex === 0 ? "-2" : inactMode.currentIndex === 1 ? "-1" : inactive.value,
        "", action.currentIndex === 0 ? "" : ["Default", "Stop", "Remove", "RemoveWithContent", "EnableSuperSeeding"][action.currentIndex - 1]].join("\x1f"))
    GridLayout {
        columns: 3
        Label { text: qsTr("Ratio") }
        ComboBox { id: ratioMode; model: [qsTr("Use global"), qsTr("Unlimited"), qsTr("Set to")] }
        SpinBox { id: ratio; from: 0; to: 100000; stepSize: 10; editable: true; enabled: ratioMode.currentIndex === 2; property real realValue: value / 100
            textFromValue: function (v) { return (v / 100).toFixed(2) }
            valueFromText: function (t) { return Math.round(parseFloat(t) * 100) } }
        Label { text: qsTr("Total seeding time (min)") }
        ComboBox { id: timeMode; model: [qsTr("Use global"), qsTr("Unlimited"), qsTr("Set to")] }
        SpinBox { id: minutes; from: 0; to: 10000000; editable: true; enabled: timeMode.currentIndex === 2 }
        Label { text: qsTr("Inactive seeding time (min)") }
        ComboBox { id: inactMode; model: [qsTr("Use global"), qsTr("Unlimited"), qsTr("Set to")] }
        SpinBox { id: inactive; from: 0; to: 10000000; editable: true; enabled: inactMode.currentIndex === 2 }
        Label { text: qsTr("When limit is reached") }
        ComboBox { id: action; Layout.columnSpan: 2; Layout.fillWidth: true; model: [qsTr("Keep current"), qsTr("Default"), qsTr("Stop"), qsTr("Remove torrent"), qsTr("Remove torrent and files"), qsTr("Enable super seeding")] }
    }
}

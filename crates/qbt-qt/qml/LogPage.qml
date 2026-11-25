import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

Item {
    id: root
    property bool active: false
    GenericTable { id: log; Component.onCompleted: log.configure("log", 4) }
    GenericTable { id: peerLog; Component.onCompleted: peerLog.configure("peerlog", 5) }
    onActiveChanged: updateWatch()
    function updateWatch() {
        AppController.unwatch("log"); AppController.unwatch("peerlog")
        if (!active) return
        AppController.watch(bar.currentIndex === 0 ? "log" : "peerlog", "")
    }
    function levelColor(t) { return t === 8 ? "#c0392b" : t === 4 ? "#d68910" : t === 2 ? "#2874a6" : "" }
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        TabBar {
            id: bar
            Layout.fillWidth: true
            onCurrentIndexChanged: root.updateWatch()
            TabButton { text: qsTr("Execution log") }
            TabButton { text: qsTr("Banned peers") }
        }
        RowLayout {
            Layout.margins: 4
            TextField { id: filter; Layout.fillWidth: true; placeholderText: qsTr("Filter"); onTextChanged: { log.setFilter(text); peerLog.setFilter(text) } }
            Button { text: qsTr("Clear"); onClicked: AppController.clearTable(bar.currentIndex === 0 ? "log" : "peerlog") }
        }
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: bar.currentIndex
            AppTable {
                id: logTable
                table: log
                sortable: false
                headers: ["#", qsTr("Time"), qsTr("Level"), qsTr("Message")]
                widths: [60, 160, 60, 1200]
                cellDelegate: Rectangle {
            clip: true
                    required property int row
                    required property int column
                    required property string display
                    implicitHeight: logTable.rowHeight
                    color: row % 2 ? logTable.palette.alternateBase : logTable.palette.base
                    Label {
                        anchors { fill: parent; leftMargin: 6 }
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                        text: column === 2 ? ({ 1: qsTr("Normal"), 2: qsTr("Info"), 4: qsTr("Warning"), 8: qsTr("Critical") })[log.cell(row, 2)] || "" : display
                        color: root.levelColor(log.cell(row, 2)) || logTable.palette.text
                    }
                }
            }
            AppTable {
                table: peerLog
                sortable: false
                headers: ["#", qsTr("Time"), qsTr("IP"), qsTr("Blocked"), qsTr("Reason")]
                widths: [60, 160, 200, 80, 600]
            }
        }
    }
}

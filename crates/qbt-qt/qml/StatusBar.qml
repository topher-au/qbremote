import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

ToolBar {
    id: root
    signal limitsRequested()
    Accessible.role: Accessible.StatusBar
    Accessible.name: qsTr("Status bar")
    readonly property bool online: AppController.connectionState === "connected"
    RowLayout {
        anchors { fill: parent; leftMargin: 8; rightMargin: 8 }
        spacing: 14
        Rectangle {
            width: 10; height: 10; radius: 5
            color: AppController.connectionState === "connected" ? "#2ecc40"
                 : AppController.connectionState === "failed" || AppController.connectionState === "disconnected" ? "#e74c3c" : "#f5a623"
            ToolTip.visible: statusHover.hovered
            ToolTip.text: AppController.statusMessage.length ? AppController.statusMessage : AppController.connectionState
            HoverHandler { id: statusHover }
        }
        Label {
            visible: AppController.connectionState === "retrying" || AppController.connectionState === "failed"
            text: AppController.statusMessage
            elide: Text.ElideRight
            Layout.maximumWidth: 420
            color: "#c0392b"
        }
        Label { visible: AppController.externalIp.length > 0; text: qsTr("External IP: %1").arg(AppController.externalIp) }
        Item { Layout.fillWidth: true }
        Label { visible: AppController.sessionPaused; text: qsTr("Session paused"); color: "#c0392b"; font.bold: true }
        Label { text: qsTr("Free space: %1").arg(AppController.freeSpace); visible: root.online }
        Label { text: qsTr("DHT: %1 nodes").arg(AppController.dhtNodes); visible: root.online }
        Label {
            visible: root.online
            text: AppController.connStatus === "connected" ? qsTr("Connected")
                : AppController.connStatus === "firewalled" ? qsTr("Firewalled") : qsTr("Disconnected")
            color: AppController.connStatus === "connected" ? "#2ecc40" : "#f5a623"
        }
        ToolButton {
            visible: root.online
            padding: 2
            text: qsTr("Alt speed")
            checkable: true
            checked: AppController.altSpeed
            onClicked: AppController.toggleAltSpeed()
            ToolTip.visible: hovered
            ToolTip.text: qsTr("Alternative speed limits")
            font.bold: AppController.altSpeed
        }
        ToolButton {
            visible: root.online
            padding: 2
            text: "↓ %1 (%2)  ↑ %3 (%4)".arg(AppController.dlSpeed).arg(AppController.dlData).arg(AppController.ulSpeed).arg(AppController.ulData)
            onClicked: root.limitsRequested()
            ToolTip.visible: hovered
            ToolTip.text: qsTr("Limits: ↓ %1  ↑ %2 — click to change").arg(AppController.dlLimit).arg(AppController.ulLimit)
        }
    }
}

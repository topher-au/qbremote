import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

Dialog {
    id: root
    title: qsTr("About qbremote")
    modal: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.Close
    property string started: ""
    onAboutToShow: { started = ""; if (AppController.supports("app", "processInfo") && AppController.connectionState === "connected") AppController.call("procInfo", "app", "processInfo", "") }
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) { if (tag === "procInfo" && ok) root.started = new Date(JSON.parse(body).launch_time * 1000).toLocaleString() }
    }
    ColumnLayout {
        spacing: 6
        Label { text: "qbremote"; font.pixelSize: 22; font.bold: true }
        Label { text: qsTr("A native remote client for qBittorrent's Web UI API.") }
        Label { text: qsTr("Not affiliated with the qBittorrent project.") ; opacity: 0.7 }
        Label { text: qsTr("Server: qBittorrent %1").arg(AppController.serverVersion || qsTr("(not connected)")); Layout.topMargin: 10 }
        Label { text: qsTr("WebAPI: %1").arg(AppController.webapiVersion || "—") }
        Label { text: qsTr("Server started: %1").arg(root.started); visible: root.started.length > 0 }
        Label { text: AppController.buildInfo; visible: text.length > 0; wrapMode: Text.Wrap; Layout.maximumWidth: 420 }
    }
}

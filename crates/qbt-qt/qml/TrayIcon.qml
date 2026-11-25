import QtQuick
import QtQuick.Controls
import Qt.labs.platform as Platform

// Optional system tray icon with completion notifications. Loaded only when the user enables it.
Item {
    id: root
    property var window: null
    function notify(title, message) {
        if (tray.available) tray.showMessage(title, message, Platform.SystemTrayIcon.Information, 6000)
    }
    Platform.SystemTrayIcon {
        id: tray
        visible: true
        icon.name: "network-transmit-receive"
        tooltip: "qbremote — ↓ " + AppController.dlSpeed + "  ↑ " + AppController.ulSpeed
        onActivated: function (reason) {
            if (!root.window) return
            if (root.window.visible && root.window.active) root.window.hide()
            else { root.window.show(); root.window.raise(); root.window.requestActivate() }
        }
        menu: Platform.Menu {
            Platform.MenuItem { text: qsTr("Show / hide"); onTriggered: tray.activated(Platform.SystemTrayIcon.Trigger) }
            Platform.MenuItem { text: qsTr("Quit"); onTriggered: Qt.quit() }
        }
    }
}

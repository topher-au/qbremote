import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Global normal and alternative speed limits (KiB/s; 0 = unlimited).
Dialog {
    id: root
    title: qsTr("Global speed limits")
    modal: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.Ok | Dialog.Cancel
    property var limits: ({})
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag === "limitsMode" && ok) { altBox.checked = body.trim() === "1"; return }
            if (tag === "limitsDl" && ok) { dl.value = Math.round(parseInt(body) / 1024); return }
            if (tag === "limitsUl" && ok) { ul.value = Math.round(parseInt(body) / 1024); return }
            if (tag !== "limits" || !ok) return
            var l = JSON.parse(body)
            dl.value = Math.round(l.dl_limit / 1024); ul.value = Math.round(l.up_limit / 1024)
            adl.value = Math.round(l.alt_dl_limit / 1024); aul.value = Math.round(l.alt_up_limit / 1024)
        }
    }
    onAboutToShow: {
        AppController.call("limitsMode", "transfer", "speedLimitsMode", "")
        if (AppController.supports("transfer", "getSpeedLimits")) AppController.call("limits", "transfer", "getSpeedLimits", "")
        else { AppController.call("limitsDl", "transfer", "downloadLimit", ""); AppController.call("limitsUl", "transfer", "uploadLimit", "") }
    }
    onAccepted: {
        AppController.call("limitsModeSet", "transfer", "setSpeedLimitsMode", JSON.stringify({ mode: altBox.checked ? 1 : 0 }))
        AppController.setGlobalLimits(dl.value * 1024, ul.value * 1024, adl.value * 1024, aul.value * 1024)
    }
    function _unused() { return 0 }
    ColumnLayout {
    CheckBox { id: altBox; text: qsTr("Use alternative speed limits") }
    GridLayout {
        columns: 3
        Label {}
        Label { text: qsTr("Download"); font.bold: true }
        Label { text: qsTr("Upload"); font.bold: true }
        Label { text: qsTr("Normal (KiB/s)") }
        SpinBox { id: dl; from: 0; to: 10000000; editable: true }
        SpinBox { id: ul; from: 0; to: 10000000; editable: true }
        Label { text: qsTr("Alternative (KiB/s)") }
        SpinBox { id: adl; from: 0; to: 10000000; editable: true; enabled: AppController.supports("transfer", "getSpeedLimits"); ToolTip.visible: hovered && !enabled; ToolTip.text: AppController.requirement("transfer", "getSpeedLimits") }
        SpinBox { id: aul; from: 0; to: 10000000; editable: true; enabled: AppController.supports("transfer", "getSpeedLimits"); ToolTip.visible: hovered && !enabled; ToolTip.text: AppController.requirement("transfer", "getSpeedLimits") }
    }
    }
}

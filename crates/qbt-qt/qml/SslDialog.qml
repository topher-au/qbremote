import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Per-torrent SSL parameters (certificate, private key, DH parameters).
Dialog {
    id: root
    property string hash: ""
    title: qsTr("SSL parameters")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(640, (parent ? parent.width : 640) - 40)
    standardButtons: Dialog.Save | Dialog.Cancel
    signal submitted(string arg)
    onAboutToShow: AppController.call("ssl", "torrents", "SSLParameters", JSON.stringify({ hash: root.hash }))
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag !== "ssl" || !ok) return
            var o = JSON.parse(body)
            cert.text = o.ssl_certificate || ""; key.text = o.ssl_private_key || ""; dh.text = o.ssl_dh_params || ""
        }
    }
    onAccepted: root.submitted([cert.text, key.text, dh.text].join("\x1f"))
    ColumnLayout {
        anchors.fill: parent
        Label { text: qsTr("Certificate (PEM)") }
        ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 90; TextArea { id: cert } }
        Label { text: qsTr("Private key (PEM)") }
        ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 90; TextArea { id: key } }
        Label { text: qsTr("DH parameters (PEM)") }
        ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 90; TextArea { id: dh } }
    }
}

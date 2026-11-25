import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Cookies the server sends when downloading torrents and RSS feeds.
Dialog {
    id: root
    title: qsTr("Manage cookies")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(820, (parent ? parent.width : 820) - 40)
    height: Math.min(460, (parent ? parent.height : 460) - 40)
    standardButtons: Dialog.Save | Dialog.Cancel
    ListModel { id: model }
    onAboutToShow: AppController.call("cookiesRaw", "app", "cookies", "")
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag !== "cookiesRaw" || !ok) return
            model.clear()
            var a = JSON.parse(body)
            for (var i = 0; i < a.length; ++i) model.append({ name: a[i].name || "", domain: a[i].domain || "", cpath: a[i].path || "/", cvalue: a[i].value || "", exp: a[i].expirationDate || 0 })
        }
    }
    onAccepted: {
        var out = []
        for (var i = 0; i < model.count; ++i) { var c = model.get(i); out.push({ name: c.name, domain: c.domain, path: c.cpath, value: c.cvalue, expirationDate: c.exp }) }
        AppController.call("cookiesSet", "app", "setCookies", JSON.stringify({ cookies: JSON.stringify(out) }))
    }
    ColumnLayout {
        anchors.fill: parent
        RowLayout {
            Label { text: qsTr("Name"); Layout.preferredWidth: 140 }
            Label { text: qsTr("Domain"); Layout.preferredWidth: 160 }
            Label { text: qsTr("Path"); Layout.preferredWidth: 80 }
            Label { text: qsTr("Value"); Layout.fillWidth: true }
        }
        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: model
            delegate: RowLayout {
                required property int index
                required property string name
                required property string domain
                required property string cpath
                required property string cvalue
                width: ListView.view.width
                TextField { Layout.preferredWidth: 140; text: name; onEditingFinished: model.setProperty(index, "name", text) }
                TextField { Layout.preferredWidth: 160; text: domain; onEditingFinished: model.setProperty(index, "domain", text) }
                TextField { Layout.preferredWidth: 80; text: cpath; onEditingFinished: model.setProperty(index, "cpath", text) }
                TextField { Layout.fillWidth: true; text: cvalue; onEditingFinished: model.setProperty(index, "cvalue", text) }
                Button { text: "✕"; onClicked: model.remove(index) }
            }
        }
        Button { text: qsTr("Add cookie"); onClicked: model.append({ name: "", domain: "", cpath: "/", cvalue: "", exp: 0 }) }
    }
}

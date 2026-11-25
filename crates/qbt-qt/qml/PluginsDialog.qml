import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

Dialog {
    id: root
    title: qsTr("Search plugins")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(760, (parent ? parent.width : 760) - 40)
    height: Math.min(460, (parent ? parent.height : 460) - 40)
    standardButtons: Dialog.Close
    GenericTable { id: plugins; Component.onCompleted: plugins.configure("plugins", 5) }
    function reload() { AppController.call("plugins", "search", "plugins", "") }
    onAboutToShow: reload()
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag.indexOf("plugin") === 0 && tag !== "plugins") reloadTimer.restart()
            if (tag.indexOf("plugin") === 0 && !ok) AppController.toast(body)
        }
    }
    Timer { id: reloadTimer; interval: 1500; onTriggered: root.reload() }
    function names() { var r = table.selectedRows(), n = []; for (var i = 0; i < r.length; ++i) n.push(plugins.cell(r[i], 0)); return n }
    ColumnLayout {
        anchors.fill: parent
        AppTable {
            id: table
            Layout.fillWidth: true
            Layout.fillHeight: true
            table: plugins
            sortable: false
            headers: [qsTr("Name"), qsTr("Full name"), qsTr("Version"), qsTr("Enabled"), qsTr("URL")]
            widths: [120, 200, 80, 70, 300]
        }
        RowLayout {
            Button { text: qsTr("Enable"); enabled: table.selectedRows().length > 0; onClicked: AppController.call("pluginEnable", "search", "enablePlugin", JSON.stringify({ names: names(), enable: true })) }
            Button { text: qsTr("Disable"); enabled: table.selectedRows().length > 0; onClicked: AppController.call("pluginDisable", "search", "enablePlugin", JSON.stringify({ names: names(), enable: false })) }
            Button { text: qsTr("Uninstall"); enabled: table.selectedRows().length > 0; onClicked: AppController.call("pluginUninstall", "search", "uninstallPlugin", JSON.stringify({ names: names() })) }
            Button { text: qsTr("Install from URL…"); onClicked: urlPrompt.open() }
            Button { text: qsTr("Check for updates"); onClicked: AppController.call("pluginUpdate", "search", "updatePlugins", "") }
        }
        Label {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            opacity: 0.7
            text: qsTr("Plugins are installed by the server, so only URLs work here. Installing a local .py file is not possible because the server cannot read files from this computer.")
        }
    }
    PromptDialog {
        id: urlPrompt
        title: qsTr("Install plugin")
        prompt: qsTr("Plugin URL(s), separated by |:")
        onSubmitted: function (t) { AppController.call("pluginInstall", "search", "installPlugin", JSON.stringify({ sources: t.split("|") })) }
    }
}

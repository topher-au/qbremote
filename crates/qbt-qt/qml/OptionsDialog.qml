import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Server options. Settings live on the *server*; only changed keys are sent.
Dialog {
    id: root
    title: qsTr("Options (server settings)")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(900, (parent ? parent.width : 900) - 40)
    height: Math.min(680, (parent ? parent.height : 680) - 40)
    property int rev: 0
    property int dirty: 0
    property string revealedKey: ""
    PrefsSchema { id: schema }

    function refreshDirty() { dirty = AppController.prefDirtyCount() }
    onAboutToShow: AppController.prefsLoad()
    Connections {
        target: AppController
        function onPrefsLoaded() { root.rev++; root.refreshDirty() }
        function onCallResult(tag, ok, body) {
            if (tag === "rotateKey" && ok) { keyDialog.key = body.replace(/"/g, ""); keyDialog.open() }
            if (tag === "ifaceAddrs" && ok) { try { var addrs = JSON.parse(body); addrModel.clear(); for (var j = 0; j < addrs.length; ++j) addrModel.append({ a: String(addrs[j]) }) } catch (e) {} }
            if (tag === "ifaces" && ok) { try { var a = JSON.parse(body); ifaceModel.clear(); for (var i = 0; i < a.length; ++i) ifaceModel.append({ n: a[i].name !== undefined ? a[i].name : String(a[i]), v: a[i].value !== undefined ? a[i].value : String(a[i]) }) } catch (e) {} }
        }
    }

    ListModel { id: ifaceModel }
    ListModel { id: addrModel }

    contentItem: RowLayout {
        spacing: 0
        ListView {
            id: pageList
            Layout.preferredWidth: 160
            Layout.fillHeight: true
            model: schema.pages
            currentIndex: 0
            clip: true
            delegate: ItemDelegate {
                required property int index
                required property var modelData
                width: ListView.view.width
                text: qsTr(modelData.title)
                highlighted: pageList.currentIndex === index
                onClicked: pageList.currentIndex = index
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Label {
                Layout.margins: 6
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                opacity: 0.7
                text: qsTr("These settings are stored on the qBittorrent server, not on this computer.")
            }
            RowLayout {
                Layout.leftMargin: 6
                visible: pageList.currentIndex === 5   // Web UI
                Button { text: qsTr("Rotate API key"); enabled: AppController.capApiKey; onClicked: AppController.call("rotateKey", "app", "rotateAPIKey", "") }
                Button { text: qsTr("Delete API key"); enabled: AppController.capApiKey; onClicked: AppController.call("delKey", "app", "deleteAPIKey", "") }
            }
            RowLayout {
                Layout.leftMargin: 6
                visible: pageList.currentIndex === 1   // Downloads
                Button { text: qsTr("Send test email"); onClicked: { AppController.prefsApply(); AppController.call("mail", "app", "sendTestEmail", "") } }
            }
            ListView {
                id: keyList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: schema.pages[pageList.currentIndex].keys
                ScrollBar.vertical: ScrollBar {}
                delegate: Item {
                    id: row
                    required property string modelData
                    readonly property string key: modelData
                    readonly property string type: { root.rev; return AppController.prefType(key) }
                    readonly property var choice: schema.choices[key]
                    readonly property bool secret: AppController.prefIsSecret(key)
                    width: ListView.view.width
                    height: 36
                    RowLayout {
                        anchors { fill: parent; leftMargin: 8; rightMargin: 14 }
                        Label { Layout.preferredWidth: 300; text: schema.label(row.key); elide: Text.ElideRight; ToolTip.visible: nameHover.hovered; ToolTip.text: row.key; HoverHandler { id: nameHover } }
                        Loader {
                            Layout.fillWidth: true
                            sourceComponent: row.type === "missing" && row.key !== "web_ui_password" ? missingC
                                : row.choice !== undefined ? comboC
                                : row.type === "bool" ? boolC
                                : row.type === "int" ? intC
                                : textC
                        }
                    }
                    Component { id: missingC; Label { text: qsTr("not provided by this server"); opacity: 0.5 } }
                    Component {
                        id: boolC
                        Item {
                            implicitHeight: boolBox.implicitHeight
                            CheckBox {
                                id: boolBox
                                anchors.left: parent.left
                                checked: { root.rev; return AppController.pref(row.key) === "true" }
                                onToggled: { AppController.prefSet(row.key, checked ? "true" : "false"); root.refreshDirty() }
                            }
                        }
                    }
                    Component {
                        id: intC
                        SpinBox {
                            editable: true
                            from: -2147483647
                            to: 2147483647
                            value: { root.rev; return parseInt(AppController.pref(row.key)) || 0 }
                            onValueModified: { AppController.prefSet(row.key, String(value)); root.refreshDirty() }
                        }
                    }
                    Component {
                        id: comboC
                        ComboBox {
                            model: row.choice
                            textRole: "1"
                            currentIndex: { root.rev; var cur = AppController.pref(row.key); for (var i = 0; i < row.choice.length; ++i) if (String(row.choice[i][0]) === cur) return i; return -1 }
                            displayText: currentIndex >= 0 ? row.choice[currentIndex][1] : AppController.pref(row.key)
                            delegate: ItemDelegate { required property int index; width: parent ? parent.width : 200; text: row.choice[index][1] }
                            onActivated: function (i) { AppController.prefSet(row.key, String(row.choice[i][0])); root.refreshDirty() }
                        }
                    }
                    Component {
                        id: textC
                        RowLayout {
                            TextField {
                                Layout.fillWidth: true
                                echoMode: row.secret && root.revealedKey !== row.key ? TextInput.Password : TextInput.Normal
                                text: { root.rev; return row.key === "web_ui_password" ? "" : AppController.pref(row.key) }
                                placeholderText: row.key === "web_ui_password" ? qsTr("(write-only; leave empty to keep)") : ""
                                onEditingFinished: { if (row.key === "web_ui_password" && text.length === 0) return; AppController.prefSet(row.key, text); root.refreshDirty() }
                            }
                            Button { visible: row.secret && row.key !== "web_ui_password"; text: root.revealedKey === row.key ? qsTr("Hide") : qsTr("Show"); onClicked: root.revealedKey = root.revealedKey === row.key ? "" : row.key }
                            Button { visible: row.key === "current_network_interface"; text: qsTr("List"); onClicked: AppController.call("ifaces", "app", "networkInterfaceList", "") }
                            ComboBox { visible: row.key === "current_network_interface" && ifaceModel.count > 0; model: ifaceModel; textRole: "n"; onActivated: function (i) { AppController.prefSet("current_network_interface", ifaceModel.get(i).v); AppController.prefSet("current_interface_name", ifaceModel.get(i).n); root.rev++; root.refreshDirty() } }
                            Button { visible: row.key === "current_interface_address"; text: qsTr("List"); onClicked: AppController.call("ifaceAddrs", "app", "networkInterfaceAddressList", JSON.stringify({ iface: AppController.pref("current_network_interface") })) }
                            ComboBox { visible: row.key === "current_interface_address" && addrModel.count > 0; model: addrModel; textRole: "a"; onActivated: function (i) { AppController.prefSet("current_interface_address", addrModel.get(i).a); root.rev++; root.refreshDirty() } }
                            Button { visible: schema.isPath(row.key); text: "…"; onClicked: { picker.key = row.key; picker.path = AppController.pref(row.key) || "/"; picker.open() } }
                        }
                    }
                }
            }
        }
    }

    RemotePathPicker {
        id: picker
        property string key: ""
        onPicked: function (p) { AppController.prefSet(key, p); root.rev++; root.refreshDirty() }
    }
    Dialog {
        id: keyDialog
        property string key: ""
        title: qsTr("New API key")
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok
        ColumnLayout {
            Label { text: qsTr("Copy this key now: it will not be shown again.") }
            TextField { id: keyField; Layout.fillWidth: true; readOnly: true; text: keyDialog.key; selectByMouse: true }
            Button { text: qsTr("Copy"); onClicked: { keyField.selectAll(); keyField.copy() } }
        }
        onClosed: key = ""
    }

    footer: DialogButtonBox {
        Label { text: root.dirty > 0 ? qsTr("%1 change(s)").arg(root.dirty) : ""; DialogButtonBox.buttonRole: DialogButtonBox.ResetRole }
        Button { text: qsTr("Apply"); enabled: root.dirty > 0; DialogButtonBox.buttonRole: DialogButtonBox.ApplyRole; onClicked: { AppController.prefsApply(); root.refreshDirty() } }
        Button { text: qsTr("OK"); highlighted: true; DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole }
        Button { text: qsTr("Cancel"); DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
        onAccepted: { AppController.prefsApply(); root.close() }
        onRejected: { AppController.prefsDiscard(); root.close() }
    }
}

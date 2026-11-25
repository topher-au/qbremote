import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.qbremote

// Connection profiles: pick or edit a server and connect.
Dialog {
    id: root
    title: qsTr("Connect to server")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(760, (parent ? parent.width : 760) - 40)
    height: Math.min(520, (parent ? parent.height : 520) - 40)
    property string testMessage: ""
    property bool testOk: false
    property string selectedName: ""
    signal connectRequested(string name, string secret)

    GenericTable { id: profiles; Component.onCompleted: { profiles.configure("profiles", 3); AppController.loadProfiles() } }

    function load(name) {
        selectedName = name
        nameField.text = name
        urlField.text = AppController.profileField(name, "url") || "http://localhost:8080/"
        var a = AppController.profileField(name, "auth")
        authBox.currentIndex = a === "apikey" ? 1 : a === "basic" ? 2 : 0
        userField.text = AppController.profileField(name, "username")
        caField.text = AppController.profileField(name, "ca_path")
        secretField.text = ""
        secretField.placeholderText = AppController.profileField(name, "has_secret") === "true" ? qsTr("(saved in keyring)") : ""
        testMessage = ""
    }
    function authName() { return ["cookie", "apikey", "basic"][authBox.currentIndex] }
    function saveCurrent() {
        if (nameField.text.length === 0) return false
        AppController.saveProfile(nameField.text, urlField.text, authName(), userField.text, secretField.text, rememberBox.checked, caField.text)
        selectedName = nameField.text
        return true
    }
    onAboutToShow: {
        var last = AppController.lastProfile()
        load(last.length ? last : "")
        if (!last.length) { nameField.text = qsTr("My server") }
    }

    Connections {
        target: AppController
        function onTestResult(ok, message) { root.testOk = ok; root.testMessage = message }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 12
        ColumnLayout {
            Layout.preferredWidth: 200
            Layout.fillHeight: true
            Label { text: qsTr("Profiles"); font.bold: true }
            ListView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                model: profiles
                delegate: ItemDelegate {
                    required property int index
                    width: ListView.view.width
                    text: profiles.cell(index, 0)
                    highlighted: profiles.cell(index, 0) === root.selectedName
                    onClicked: root.load(profiles.cell(index, 0))
                }
            }
            RowLayout {
                Button { text: qsTr("New"); onClicked: { root.load(""); nameField.text = qsTr("New server"); nameField.forceActiveFocus() } }
                Button { text: qsTr("Delete"); enabled: root.selectedName.length > 0; onClicked: { AppController.deleteProfile(root.selectedName); root.load("") } }
            }
        }
        GridLayout {
            columns: 2
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignTop
            columnSpacing: 10
            Label { text: qsTr("Name") }
            TextField { id: nameField; Layout.fillWidth: true }
            Label { text: qsTr("URL") }
            TextField { id: urlField; Layout.fillWidth: true; placeholderText: "https://host:8080/" }
            Label { text: qsTr("Authentication") }
            ComboBox { id: authBox; Layout.fillWidth: true; model: [qsTr("Username & password"), qsTr("API key (WebAPI 2.14.1+)"), qsTr("HTTP Basic (WebAPI 2.15.0+)")] }
            Label { text: qsTr("Username"); visible: authBox.currentIndex !== 1 }
            TextField { id: userField; Layout.fillWidth: true; visible: authBox.currentIndex !== 1 }
            Label { text: authBox.currentIndex === 1 ? qsTr("API key") : qsTr("Password") }
            TextField { id: secretField; Layout.fillWidth: true; echoMode: TextInput.Password }
            Label { text: qsTr("CA certificate") }
            RowLayout {
                Layout.fillWidth: true
                TextField { id: caField; Layout.fillWidth: true; placeholderText: qsTr("optional PEM file for self-signed servers") }
                Button { text: "…"; onClicked: caDialog.open() }
            }
            Item { width: 1 }
            CheckBox { id: rememberBox; text: qsTr("Remember secret in the system keyring"); checked: true }
            Item { width: 1 }
            Label {
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                visible: root.testMessage.length > 0
                text: root.testMessage
                color: root.testOk ? "#2e8b57" : "#c0392b"
            }
        }
    }

    FileDialog { id: caDialog; title: qsTr("CA certificate"); nameFilters: ["PEM (*.pem *.crt)", "All (*)"]; onAccepted: caField.text = AppController.localPath(selectedFile.toString()) }

    footer: DialogButtonBox {
        Button { text: qsTr("Test connection"); DialogButtonBox.buttonRole: DialogButtonBox.ActionRole; onClicked: { root.testMessage = qsTr("Testing…"); root.testOk = true; AppController.testConnection(urlField.text, root.authName(), userField.text, secretField.text, caField.text) } }
        Button { text: qsTr("Save"); DialogButtonBox.buttonRole: DialogButtonBox.ActionRole; onClicked: root.saveCurrent() }
        Button { text: qsTr("Connect"); highlighted: true; DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole }
        Button { text: qsTr("Cancel"); DialogButtonBox.buttonRole: DialogButtonBox.RejectRole }
        onAccepted: { if (root.saveCurrent()) root.connectRequested(nameField.text, secretField.text) }
        onRejected: root.close()
    }
}

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Browse the *server's* file system (app/getDirectoryContent). Never uses local file dialogs.
Dialog {
    id: root
    property string path: "/"
    property string mode: "dirs"
    property string freeSpace: ""
    signal picked(string path)
    title: qsTr("Choose a folder on the server")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(620, (parent ? parent.width : 620) - 40)
    height: Math.min(520, (parent ? parent.height : 520) - 40)
    standardButtons: Dialog.Ok | Dialog.Cancel

    GenericTable { id: dir; Component.onCompleted: dir.configure("dir", 4) }

    function load(p) {
        root.path = p
        pathField.text = p
        AppController.call("dir", "app", "getDirectoryContent", JSON.stringify({ dirPath: p, mode: root.mode }))
        AppController.call("space", "app", "getFreeSpaceAtPath", JSON.stringify({ path: p }))
    }
    function parentOf(p) {
        var sep = p.indexOf("\\") >= 0 && p.indexOf("/") < 0 ? "\\" : "/"
        var t = p.length > 1 && (p.endsWith("/") || p.endsWith("\\")) ? p.slice(0, -1) : p
        var i = t.lastIndexOf(sep)
        return i <= 0 ? (sep === "/" ? "/" : t.slice(0, i + 1)) : t.slice(0, i)
    }
    onAboutToShow: load(root.path.length ? root.path : "/")
    onAccepted: root.picked(pathField.text)

    Connections {
        target: AppController
        function onCallResult(tag, ok, body) { if (tag === "space" && ok) root.freeSpace = AppController.formatSize(parseInt(body)) }
    }

    ColumnLayout {
        anchors.fill: parent
        RowLayout {
            Button { text: "↑"; onClicked: root.load(root.parentOf(pathField.text)) }
            TextField { id: pathField; Layout.fillWidth: true; onAccepted: root.load(text) }
            Button { text: qsTr("Go"); onClicked: root.load(pathField.text) }
        }
        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: dir
            delegate: ItemDelegate {
                required property int index
                width: ListView.view.width
                text: (dir.cell(index, 1) === "dir" ? "📁 " : "📄 ") + dir.cell(index, 0)
                onClicked: pathField.text = dir.cell(index, 3)
                onDoubleClicked: if (dir.cell(index, 1) === "dir") root.load(dir.cell(index, 3))
            }
            ScrollBar.vertical: ScrollBar {}
        }
        Label { text: root.freeSpace.length ? qsTr("Free space: %1").arg(root.freeSpace) : ""; opacity: 0.7 }
    }
}

import QtQuick
import QtQuick.Controls
import org.qbremote

// Context menu for the selected torrents.
Menu {
    id: root
    signal action(string name, string arg)
    signal dialogRequested(string name)

    GenericTable { id: cats; Component.onCompleted: cats.configure("category_names", 1) }
    GenericTable { id: tags; Component.onCompleted: tags.configure("tag_names", 1) }

    MenuItem { text: qsTr("Start"); onTriggered: root.action("start", "") }
    MenuItem { text: qsTr("Stop"); onTriggered: root.action("stop", "") }
    MenuItem { text: qsTr("Force start"); onTriggered: root.action("forceStart", "true") }
    MenuItem { text: qsTr("Delete…"); onTriggered: root.dialogRequested("delete") }
    MenuSeparator {}
    MenuItem { text: qsTr("Set location…"); onTriggered: root.dialogRequested("location") }
    MenuItem { text: qsTr("Set save path…"); onTriggered: root.dialogRequested("savePath") }
    MenuItem { text: qsTr("Set incomplete path…"); onTriggered: root.dialogRequested("downloadPath") }
    MenuItem { text: qsTr("Rename…"); onTriggered: root.dialogRequested("rename") }
    MenuItem { text: qsTr("Edit comment…"); onTriggered: root.dialogRequested("comment") }
    Menu {
        id: catMenu
        title: qsTr("Category")
        MenuItem { text: qsTr("New…"); onTriggered: root.dialogRequested("newCategory") }
        MenuItem { text: qsTr("Reset"); onTriggered: root.action("setCategory", "") }
        MenuSeparator {}
        Instantiator {
            model: cats
            delegate: MenuItem {
                text: model.display
                onTriggered: root.action("setCategory", model.display)
            }
            onObjectAdded: function (i, o) { catMenu.insertItem(i + 3, o) }
            onObjectRemoved: function (i, o) { catMenu.removeItem(o) }
        }
    }
    Menu {
        id: tagMenu
        title: qsTr("Tags")
        MenuItem { text: qsTr("Add…"); onTriggered: root.dialogRequested("addTags") }
        MenuItem { text: qsTr("Clear all"); onTriggered: root.action("setTags", "") }
        MenuSeparator {}
        Instantiator {
            model: tags
            delegate: MenuItem {
                text: model.display
                onTriggered: root.action("addTags", model.display)
            }
            onObjectAdded: function (i, o) { tagMenu.insertItem(i + 3, o) }
            onObjectRemoved: function (i, o) { tagMenu.removeItem(o) }
        }
    }
    Menu {
        id: untagMenu
        title: qsTr("Remove tag")
        Instantiator {
            model: tags
            delegate: MenuItem {
                text: model.display
                onTriggered: root.action("removeTags", model.display)
            }
            onObjectAdded: function (i, o) { untagMenu.insertItem(i, o) }
            onObjectRemoved: function (i, o) { untagMenu.removeItem(o) }
        }
    }
    MenuItem { text: qsTr("Automatic torrent management"); checkable: true; onTriggered: root.action("setAutoManagement", checked ? "true" : "false") }
    MenuSeparator {}
    MenuItem { text: qsTr("Limit download rate…"); onTriggered: root.dialogRequested("dlLimit") }
    MenuItem { text: qsTr("Limit upload rate…"); onTriggered: root.dialogRequested("ulLimit") }
    MenuItem { text: qsTr("Limit share ratio…"); onTriggered: root.dialogRequested("shareLimits") }
    MenuItem { text: qsTr("Super seeding"); checkable: true; onTriggered: root.action("setSuperSeeding", checked ? "true" : "false") }
    MenuItem { text: qsTr("Sequential download"); onTriggered: root.action("toggleSequentialDownload", "") }
    MenuItem { text: qsTr("Download first and last pieces first"); onTriggered: root.action("toggleFirstLastPiecePrio", "") }
    MenuSeparator {}
    MenuItem { text: qsTr("Force recheck"); onTriggered: root.action("recheck", "") }
    MenuItem { text: qsTr("Force reannounce"); onTriggered: root.action("reannounce", "") }
    Menu {
        title: qsTr("Queue")
        MenuItem { text: qsTr("Move to top"); onTriggered: root.action("topPrio", "") }
        MenuItem { text: qsTr("Move up"); onTriggered: root.action("increasePrio", "") }
        MenuItem { text: qsTr("Move down"); onTriggered: root.action("decreasePrio", "") }
        MenuItem { text: qsTr("Move to bottom"); onTriggered: root.action("bottomPrio", "") }
    }
    Menu {
        title: qsTr("Copy")
        MenuItem { text: qsTr("Name"); onTriggered: root.dialogRequested("copyName") }
        MenuItem { text: qsTr("Info hash v1"); onTriggered: root.dialogRequested("copyHash1") }
        MenuItem { text: qsTr("Info hash v2"); onTriggered: root.dialogRequested("copyHash2") }
        MenuItem { text: qsTr("Magnet link"); onTriggered: root.dialogRequested("copyMagnet") }
        MenuItem { text: qsTr("Comment"); onTriggered: root.dialogRequested("copyComment") }
        MenuItem { text: qsTr("Content path"); onTriggered: root.dialogRequested("copyPath") }
    }
    MenuItem { text: qsTr("Export .torrent…"); enabled: AppController.supports("torrents", "export"); onTriggered: root.dialogRequested("export") }
    MenuItem {
        text: qsTr("SSL parameters…")
        enabled: AppController.supports("torrents", "SSLParameters")
        onTriggered: root.dialogRequested("ssl")
    }
}

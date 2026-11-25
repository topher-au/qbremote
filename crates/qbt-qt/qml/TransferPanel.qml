import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// The transfer list: one row per torrent.
Item {
    id: root
    property alias model: transfers
    property alias table: table
    signal action(string name, string arg)       // simple actions on the selection
    signal dialogRequested(string name)          // dialogs needing the selection
    signal filesDropped(var urls)
    signal textDropped(string text)
    signal selectionChanged2()

    TransferListModel { id: transfers }

    function selectedHashes() {
        var rows = table.selectedRows(), out = []
        for (var i = 0; i < rows.length; ++i) out.push(transfers.hashAt(rows[i]))
        return out
    }
    function selectedHash() { var h = selectedHashes(); return h.length === 1 ? h[0] : "" }
    function selectedNames() {
        var rows = table.selectedRows(), out = []
        for (var i = 0; i < rows.length; ++i) out.push(transfers.textAt(rows[i], "name"))
        return out
    }
    function hiddenFromSetting() {
        var s = AppController.setting("transferColumns")
        var keys = s.length ? s.split(",") : null
        var hidden = []
        for (var c = 0; c < table.view.columns; ++c) {
            var visible = keys ? keys.indexOf(transfers.columnKey(c)) >= 0 : transfers.columnDefaultVisible(c)
            if (!visible) hidden.push(c)
        }
        return hidden
    }
    // Column widths persist as "key:width,key:width".
    function widthsFromSetting() {
        var out = {}, s = AppController.setting("columnWidths")
        if (!s.length) return out
        var parts = s.split(",")
        for (var i = 0; i < parts.length; ++i) {
            var kv = parts[i].split(":")
            for (var c = 0; c < table.view.columns; ++c) if (transfers.columnKey(c) === kv[0]) out[c] = parseInt(kv[1])
        }
        return out
    }
    function saveWidths() {
        var parts = [], u = table.userWidths
        for (var c in u) parts.push(transfers.columnKey(parseInt(c)) + ":" + Math.round(u[c]))
        AppController.setSetting("columnWidths", parts.join(","))
        AppController.syncUiStore()
    }
    // Column order persists as the keys in visual order.
    function orderFromSetting() {
        var s = AppController.setting("columnOrder")
        if (!s.length) return []
        var out = []
        var keys = s.split(",")
        for (var i = 0; i < keys.length; ++i)
            for (var c = 0; c < table.view.columns; ++c) if (transfers.columnKey(c) === keys[i]) out.push(c)
        return out
    }
    function saveOrder() {
        var keys = []
        for (var v = 0; v < table.order.length; ++v) keys.push(transfers.columnKey(table.order[v]))
        AppController.setSetting("columnOrder", keys.join(","))
        AppController.syncUiStore()
    }
    function reloadLayout() {
        table.hiddenColumns = hiddenFromSetting()
        table.userWidths = widthsFromSetting()
        table.applyOrder(orderFromSetting())
        table.layoutRev++
        table.view.forceLayout()
    }
    Connections { target: AppController; function onUiStateLoaded() { root.reloadLayout() } }
    function toggleColumn(c) {
        var h = table.hiddenColumns.slice(), i = h.indexOf(c)
        if (i >= 0) h.splice(i, 1); else h.push(c)
        table.hiddenColumns = h
        var vis = []
        for (var k = 0; k < table.view.columns; ++k) if (h.indexOf(k) < 0) vis.push(transfers.columnKey(k))
        AppController.setSetting("transferColumns", vis.join(","))
        AppController.syncUiStore()
    }
    property alias contextMenu: ctxMenu

    AppTable {
        id: table
        anchors.fill: parent
        table: transfers
        accessibleName: qsTr("Transfer list")
        sortColumn: transfers.sortColumn()
        sortDescending: transfers.sortDescending()
        widths: [40, 320, 90, 90, 130, 130, 80, 80, 90, 90, 70, 60, 80, 110, 110, 150, 150, 200, 80, 80, 90, 90, 100, 100, 90, 90, 90, 220, 220, 90, 80, 150, 150, 80, 90, 70, 300, 300, 250]
        Component.onCompleted: {
            var h = []
            for (var c = 0; c < view.columns; ++c) h.push(transfers.header(c))
            headers = h
            hiddenColumns = hiddenFromSetting()
            userWidths = widthsFromSetting()
            applyOrder(orderFromSetting())
        }
        reorderable: true
        onOrderCommitted: root.saveOrder()
        onWidthsCommitted: root.saveWidths()
        onHeaderSortRequested: function (c, d) {
            transfers.sortBy(c, d)
            sortColumn = c
            sortDescending = d
        }
        onHeaderMenuRequested: colMenu.popup()
        onActivated: function (row) {
            if (AppController.setting("doubleClickAction") === "properties") root.dialogRequested("properties")
            else root.action("toggle", "")
        }
        onContextMenuRequested: function (row) {
            if (!isRowSelected(row)) selectRow(row)
            ctxMenu.popup()
        }
        cellDelegate: Rectangle {
            clip: true
            required property bool selected
            required property int row
            required property int column
            required property string display
            required property var value
            implicitWidth: 100
            implicitHeight: table.rowHeight
            color: selected ? table.palette.highlight : (row % 2 ? table.palette.alternateBase : table.palette.base)
            readonly property bool isProgress: transfers.columnKey(column) === "progress"
            Rectangle {
                visible: parent.isProgress
                anchors { left: parent.left; top: parent.top; bottom: parent.bottom; margins: 2 }
                width: Math.max(0, (parent.width - 4) * (typeof value === "number" ? value : 0))
                color: parent.selected ? Qt.rgba(1, 1, 1, 0.35) : "#5aa9e6"
                opacity: 0.8
            }
            Label {
                anchors { fill: parent; leftMargin: 6; rightMargin: 6 }
                text: display
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: parent.isProgress ? Text.AlignHCenter : Text.AlignLeft
                color: selected ? table.palette.highlightedText : table.palette.text
            }
            TapHandler { acceptedButtons: Qt.RightButton; onTapped: table.contextMenuRequested(row) }
            TapHandler { acceptedButtons: Qt.LeftButton; onDoubleTapped: table.activated(row) }
        }
    }

    Connections {
        target: table.view.selectionModel
        function onSelectionChanged() { root.selectionChanged2() }
    }

    // Tell the model which rows are on screen so live updates only repaint those.
    function reportVisibleRows() { transfers.setVisibleRows(table.view.topRow, table.view.bottomRow) }
    Connections {
        target: table.view
        function onContentYChanged() { root.reportVisibleRows() }
        function onRowsChanged() { root.reportVisibleRows() }
        function onHeightChanged() { root.reportVisibleRows() }
    }
    Timer { interval: 250; running: true; repeat: true; onTriggered: root.reportVisibleRows() }

    Menu {
        id: colMenu
        Repeater {
            model: Math.max(0, table.view.columns)
            MenuItem {
                required property int index
                text: transfers.header(index)
                checkable: true
                checked: table.hiddenColumns.indexOf(index) < 0
                onTriggered: root.toggleColumn(index)
                enabled: index !== 1
            }
        }
    }

    TorrentMenu {
        id: ctxMenu
        onAction: function (name, arg) { root.action(name, arg) }
        onDialogRequested: function (name) { root.dialogRequested(name) }
    }

    DropArea {
        anchors.fill: parent
        onDropped: function (drop) {
            if (drop.hasUrls) root.filesDropped(drop.urls)
            else if (drop.hasText) root.textDropped(drop.text)
        }
    }
}

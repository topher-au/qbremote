import QtQuick
import QtQuick.Controls
import org.qbremote

// A sortable, selectable table over a GenericTable (or any QAbstractTableModel).
Control {
    id: appTable
    property string accessibleName: ""
    Accessible.role: Accessible.Table
    Accessible.name: accessibleName
    property var headers: []
    property var widths: []
    property var table: null
    property alias view: tv
    property bool sortable: true
    property int sortColumn: -1
    property bool sortDescending: false
    property var hiddenColumns: []
    property Component cellDelegate: defaultCell
    property int rowHeight: 24
    signal activated(int row)
    signal contextMenuRequested(int row)
    signal headerSortRequested(int column, bool descending)

    function selectedRows() {
        var seen = {}
        var idx = tv.selectionModel.selectedIndexes
        for (var i = 0; i < idx.length; ++i) seen[idx[i].row] = true
        return Object.keys(seen).map(Number).sort(function (a, b) { return a - b })
    }
    function selectRow(row) {
        tv.selectionModel.clearSelection()
        tv.selectionModel.select(tv.index(row, 0), ItemSelectionModel.Select | ItemSelectionModel.Rows)
    }
    function isRowSelected(row) { return tv.selectionModel.isRowSelected(row, tv.rootIndex) }
    function widthOf(c) {
        if (hiddenColumns.indexOf(c) >= 0) return 0
        if (userWidths[c] !== undefined) return userWidths[c]
        return widths[c] !== undefined ? widths[c] : 120
    }
    onHiddenColumnsChanged: { layoutRev++; tv.forceLayout() }
    onWidthsChanged: { layoutRev++; tv.forceLayout() }

    background: Rectangle { color: appTable.palette.base; border.color: appTable.palette.mid; border.width: 1 }

    property var userWidths: ({})
    // Visual order of columns: order[visualPosition] = model column. Empty means unchanged.
    property var order: []
    property bool reorderable: false
    signal orderCommitted()
    function logicalAt(v) { return order.length > v ? order[v] : v }
    function identityOrder() { var o = []; for (var i = 0; i < headers.length; ++i) o.push(i); return o }
    // Move the column at visual position `from` to visual position `to`.
    function moveVisual(from, to) {
        if (!reorderable || from === to || from < 0 || to < 0 || to >= headers.length) return
        tv.moveColumn(from, to)
        var o = order.length ? order.slice() : identityOrder()
        o.splice(to, 0, o.splice(from, 1)[0])
        order = o
        layoutRev++
        orderCommitted()
    }
    // Arrange columns so `wanted` (model column indexes, left to right) comes first, in that order.
    function applyOrder(wanted) {
        if (!reorderable) return
        var o = order.length ? order.slice() : identityOrder()
        var pos = 0
        for (var i = 0; i < wanted.length; ++i) {
            var j = o.indexOf(wanted[i])
            if (j < 0) continue
            if (j !== pos) { tv.moveColumn(j, pos); o.splice(pos, 0, o.splice(j, 1)[0]) }
            ++pos
        }
        order = o
        layoutRev++
    }
    // Visual position under x (in header coordinates), skipping hidden (zero-width) columns.
    function visualAt(x) {
        var acc = 0
        for (var v = 0; v < headers.length; ++v) {
            var w = widthOf(logicalAt(v))
            if (x < acc + w) return v
            acc += w
        }
        return headers.length - 1
    }
    property int layoutRev: 0
    signal headerMenuRequested(int column)
    signal widthsCommitted()

    // Custom header: a row of cells scrolled in step with the table, with sort indicators,
    // drag-to-resize handles and a context-menu hook.
    Item {
        id: hh
        height: 26
        anchors { left: parent.left; top: parent.top; right: parent.right }
        clip: true
        Row {
            x: -tv.contentX
            Repeater {
                model: appTable.headers.length
                delegate: Rectangle {
                    id: cell
                    required property int index                       // visual position
                    readonly property int col: { appTable.layoutRev; return appTable.logicalAt(index) }   // model column
                    width: { appTable.layoutRev; return appTable.widthOf(col) }
                    height: hh.height
                    visible: width > 0
                    color: cellMouse.containsMouse ? Qt.lighter(appTable.palette.button, 1.1) : appTable.palette.button
                    border.color: appTable.palette.mid
                    border.width: 1
                    Label {
                        anchors { fill: parent; leftMargin: 6; rightMargin: 16 }
                        text: appTable.headers[cell.col] || ""
                        elide: Text.ElideRight
                        verticalAlignment: Text.AlignVCenter
                        font.bold: appTable.sortColumn === cell.col
                        color: appTable.palette.buttonText
                    }
                    Label {
                        visible: appTable.sortColumn === cell.col
                        anchors { right: parent.right; rightMargin: 8; verticalCenter: parent.verticalCenter }
                        text: appTable.sortDescending ? "▼" : "▲"
                        font.pixelSize: 9
                        color: appTable.palette.buttonText
                    }
                    // Drag a header cell to reorder (only on tables that opt in).
                    DragHandler {
                        id: reorder
                        enabled: appTable.reorderable
                        target: null
                        yAxis.enabled: false
                        dragThreshold: 12
                        onActiveChanged: {
                            cell.opacity = active ? 0.6 : 1.0
                            if (!active) {
                                var from = cell.index
                                var to = appTable.visualAt(cell.x + cell.width / 2 + translation.x)
                                appTable.moveVisual(from, to)
                            }
                        }
                    }
                    MouseArea {
                        id: cellMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton | Qt.RightButton
                        onClicked: function (m) {
                            if (m.button === Qt.RightButton) { appTable.headerMenuRequested(cell.col); return }
                            if (!appTable.sortable) return
                            appTable.headerSortRequested(cell.col, appTable.sortColumn === cell.col ? !appTable.sortDescending : false)
                        }
                    }
                    Rectangle {
                        // resize handle
                        width: 6
                        height: parent.height
                        anchors.right: parent.right
                        color: "transparent"
                        property real startWidth: 0
                        HoverHandler { cursorShape: Qt.SplitHCursor }
                        DragHandler {
                            id: drag
                            target: null
                            yAxis.enabled: false
                            onActiveChanged: { if (active) parent.startWidth = cell.width; else appTable.widthsCommitted() }
                            onTranslationChanged: {
                                if (!active) return
                                var w = Math.max(30, parent.startWidth + translation.x)
                                var u = appTable.userWidths
                                u[cell.col] = w
                                appTable.userWidths = u
                                appTable.layoutRev++
                                tv.forceLayout()
                            }
                        }
                    }
                }
            }
        }
    }


    TableView {
        id: tv
        anchors { left: parent.left; right: parent.right; top: hh.bottom; bottom: parent.bottom }
        clip: true
        model: appTable.table
        boundsBehavior: Flickable.StopAtBounds
        reuseItems: true
        // TableView asks for widths by visual position; ours are keyed by model column.
        columnWidthProvider: function (v) { return appTable.widthOf(appTable.logicalAt(v)) }
        rowHeightProvider: function () { return appTable.rowHeight }
        selectionModel: ItemSelectionModel {}
        selectionBehavior: TableView.SelectRows
        selectionMode: TableView.ExtendedSelection
        delegate: appTable.cellDelegate
        ScrollBar.vertical: ScrollBar {}
        ScrollBar.horizontal: ScrollBar {}
    }

    Component {
        id: defaultCell
        Rectangle {
            clip: true
            required property bool selected
            required property int row
            required property int column
            required property string display
            implicitWidth: 100
            implicitHeight: appTable.rowHeight
            color: selected ? appTable.palette.highlight : (row % 2 ? appTable.palette.alternateBase : appTable.palette.base)
            Label {
                anchors { fill: parent; leftMargin: 6; rightMargin: 6 }
                text: display
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
                color: selected ? appTable.palette.highlightedText : appTable.palette.text
            }
            TapHandler {
                acceptedButtons: Qt.RightButton
                onTapped: appTable.contextMenuRequested(row)
            }
            TapHandler {
                acceptedButtons: Qt.LeftButton
                onDoubleTapped: appTable.activated(row)
            }
        }
    }
}

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

// Filter sidebar: status, categories, tags, trackers, each with a count.
Pane {
    id: root
    signal filterSelected(string kind, string key)
    signal categoryMenuRequested(string key)
    signal tagMenuRequested(string key)
    padding: 0
    Accessible.role: Accessible.Pane
    Accessible.name: qsTr("Filters")
    property string currentId: "status:all"
    property var collapsed: ({})

    GenericTable { id: sidebar; Component.onCompleted: sidebar.configure("sidebar", 5) }

    ListView {
        id: list
        anchors.fill: parent
        clip: true
        model: sidebar
        ScrollBar.vertical: ScrollBar {}
        delegate: ItemDelegate {
            id: item
            required property int index
            required property string key      // "kind:key"
            // `rev` makes the cell() reads re-run whenever the model reports a data change.
            property int rev: 0
            readonly property string kind: { rev; var v = sidebar.cell(index, 0); return v === undefined ? "" : v }
            readonly property string itemKey: { rev; var v = sidebar.cell(index, 1); return v === undefined ? "" : v }
            readonly property string label: { rev; var v = sidebar.cell(index, 2); return v === undefined ? "" : v }
            readonly property var count: { rev; var v = sidebar.cell(index, 3); return v === undefined ? null : v }
            readonly property int depth: { rev; var v = sidebar.cell(index, 4); return v === undefined ? 0 : v }
            Connections {
                target: sidebar
                function onDataChanged() { item.rev++ }
            }
            readonly property bool isHeader: kind === "header"
            width: ListView.view.width
            height: visibleRow ? 26 : 0
            visible: visibleRow
            readonly property bool visibleRow: isHeader || !root.collapsed[sectionOf(index)]
            highlighted: !isHeader && root.currentId === key
            leftPadding: 8 + Math.max(0, depth - 1) * 14
            function sectionOf(i) {
                for (var j = i; j >= 0; --j) if (sidebar.cell(j, 0) === "header") return sidebar.cell(j, 1)
                return ""
            }
            contentItem: RowLayout {
                spacing: 6
                Label {
                    visible: item.isHeader
                    text: root.collapsed[item.itemKey] ? "▸" : "▾"
                }
                Label {
                    Layout.fillWidth: true
                    text: item.label
                    elide: Text.ElideRight
                    font.bold: item.isHeader
                }
                Label {
                    visible: !item.isHeader && item.count !== undefined && item.count !== null
                    text: "(" + item.count + ")"
                    opacity: 0.7
                }
            }
            onClicked: {
                if (isHeader) {
                    var c = root.collapsed
                    c[itemKey] = !c[itemKey]
                    root.collapsed = c
                    list.forceLayout()
                    return
                }
                root.currentId = key
                root.filterSelected(kind, itemKey)
            }
            TapHandler {
                acceptedButtons: Qt.RightButton
                onTapped: {
                    if (item.kind === "category") root.categoryMenuRequested(item.itemKey)
                    else if (item.kind === "tag") root.tagMenuRequested(item.itemKey)
                }
            }
        }
    }
}

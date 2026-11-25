import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.qbremote

// RSS auto-download rules: list on the left, every rule field on the right.
Dialog {
    id: root
    title: qsTr("RSS downloader")
    modal: true
    anchors.centerIn: Overlay.overlay
    width: Math.min(980, (parent ? parent.width : 980) - 40)
    height: Math.min(680, (parent ? parent.height : 680) - 40)
    standardButtons: Dialog.Close
    property var rules: ({})
    property string current: ""
    property string matches: ""
    GenericTable { id: ruleList; Component.onCompleted: ruleList.configure("rss_rules", 2) }

    function reload() { AppController.call("rulesRaw", "rss", "rules", "") }
    onAboutToShow: reload()
    function load(name) {
        current = name
        var d = rules[name] || {}
        enabled.checked = d.enabled !== false
        mustContain.text = d.mustContain || ""
        mustNot.text = d.mustNotContain || ""
        useRegex.checked = !!d.useRegex
        episode.text = d.episodeFilter || ""
        smart.checked = !!d.smartFilter
        ignoreDays.value = d.ignoreDays || 0
        paused.currentIndex = d.addPaused === true ? 1 : d.addPaused === false ? 2 : 0
        category.text = d.assignedCategory || ""
        savePath.text = d.savePath || ""
        layout.currentIndex = d.torrentContentLayout === "Original" ? 1 : d.torrentContentLayout === "Subfolder" ? 2 : d.torrentContentLayout === "NoSubfolder" ? 3 : 0
        feeds.text = (d.affectedFeeds || []).join("\n")
        matches = ""
    }
    function def() {
        var d = JSON.parse(JSON.stringify(rules[current] || {}))
        d.enabled = enabled.checked; d.mustContain = mustContain.text; d.mustNotContain = mustNot.text
        d.useRegex = useRegex.checked; d.episodeFilter = episode.text; d.smartFilter = smart.checked
        d.ignoreDays = ignoreDays.value
        d.addPaused = paused.currentIndex === 0 ? null : paused.currentIndex === 1
        d.assignedCategory = category.text; d.savePath = savePath.text
        d.torrentContentLayout = layout.currentIndex === 0 ? null : ["Original", "Subfolder", "NoSubfolder"][layout.currentIndex - 1]
        d.affectedFeeds = feeds.text.split("\n").map(function (s) { return s.trim() }).filter(function (s) { return s.length })
        return d
    }
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (tag === "rulesRaw" && ok) { root.rules = JSON.parse(body); AppController.call("rules", "rss", "rules", ""); if (root.current && root.rules[root.current]) root.load(root.current) }
            else if (tag === "matching" && ok) {
                var o = JSON.parse(body), out = []
                for (var f in o) { out.push(f + ":"); for (var i = 0; i < o[f].length; ++i) out.push("  " + o[f][i]) }
                root.matches = out.join("\n") || qsTr("No matching articles")
            } else if (tag.indexOf("rule") === 0 && tag !== "rules" && tag !== "rulesRaw") reloadTimer.restart()
        }
    }
    Timer { id: reloadTimer; interval: 300; onTriggered: root.reload() }

    RowLayout {
        anchors.fill: parent
        ColumnLayout {
            Layout.preferredWidth: 220
            Layout.fillHeight: true
            AppTable {
                id: list
                Layout.fillWidth: true
                Layout.fillHeight: true
                table: ruleList
                sortable: false
                headers: [qsTr("Rule"), ""]
                widths: [200, 0]
                onActivated: function (r) { root.load(ruleList.cell(r, 0)) }
                Connections { target: list.view.selectionModel; function onSelectionChanged() { var s = list.selectedRows(); if (s.length === 1) root.load(ruleList.cell(s[0], 0)) } }
            }
            RowLayout {
                Button { text: qsTr("New…"); onClicked: newPrompt.open() }
                Button { text: qsTr("Clone…"); enabled: root.current.length > 0; onClicked: clonePrompt.open() }
            }
            RowLayout {
                Button { text: qsTr("Rename…"); enabled: root.current.length > 0; onClicked: renamePrompt.open() }
                Button { text: qsTr("Delete"); enabled: root.current.length > 0; onClicked: { AppController.call("ruleDel", "rss", "removeRule", JSON.stringify({ ruleName: root.current })); root.current = "" } }
            }
            RowLayout {
                Button { text: qsTr("Export…"); enabled: AppController.supports("rss", "exportRules"); ToolTip.visible: hovered && !enabled; ToolTip.text: AppController.requirement("rss", "exportRules"); onClicked: exportDialog.open() }
                Button { text: qsTr("Import…"); enabled: AppController.supports("rss", "importRules"); ToolTip.visible: hovered && !enabled; ToolTip.text: AppController.requirement("rss", "importRules"); onClicked: importDialog.open() }
            }
        }
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            enabled: root.current.length > 0
            contentWidth: availableWidth
            ColumnLayout {
                width: parent.width
                CheckBox { id: enabled; text: qsTr("Enable rule") }
                GridLayout {
                    columns: 2
                    Layout.fillWidth: true
                    Label { text: qsTr("Must contain") }
                    TextField { id: mustContain; Layout.fillWidth: true }
                    Label { text: qsTr("Must not contain") }
                    TextField { id: mustNot; Layout.fillWidth: true }
                    Item {}
                    CheckBox { id: useRegex; text: qsTr("Use regular expressions") }
                    Label { text: qsTr("Episode filter") }
                    TextField { id: episode; Layout.fillWidth: true; placeholderText: "1x01-;2x03" }
                    Item {}
                    CheckBox { id: smart; text: qsTr("Smart episode filter") }
                    Label { text: qsTr("Ignore matches for (days)") }
                    SpinBox { id: ignoreDays; from: 0; to: 100000; editable: true }
                    Label { text: qsTr("Add paused") }
                    ComboBox { id: paused; model: [qsTr("Default"), qsTr("Yes"), qsTr("No")] }
                    Label { text: qsTr("Category") }
                    TextField { id: category; Layout.fillWidth: true }
                    Label { text: qsTr("Save to (server path)") }
                    RowLayout {
                        Layout.fillWidth: true
                        TextField { id: savePath; Layout.fillWidth: true }
                        Button { text: "…"; onClicked: { picker.path = savePath.text || "/"; picker.open() } }
                    }
                    Label { text: qsTr("Content layout") }
                    ComboBox { id: layout; model: [qsTr("Default"), qsTr("Original"), qsTr("Create subfolder"), qsTr("Don't create subfolder")] }
                    Label { text: qsTr("Feeds (URLs, one per line)") }
                    ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 70; TextArea { id: feeds } }
                }
                RowLayout {
                    Button { text: qsTr("Save rule"); highlighted: true; onClicked: AppController.call("ruleSave", "rss", "setRule", JSON.stringify({ ruleName: root.current, ruleDef: JSON.stringify(root.def()) })) }
                    Button { text: qsTr("Preview matching articles"); onClicked: { AppController.call("ruleSave2", "rss", "setRule", JSON.stringify({ ruleName: root.current, ruleDef: JSON.stringify(root.def()) })); matchTimer.start() } }
                }
                Timer { id: matchTimer; interval: 400; onTriggered: AppController.call("matching", "rss", "matchingArticles", JSON.stringify({ ruleName: root.current })) }
                ScrollView { Layout.fillWidth: true; Layout.preferredHeight: 140; TextArea { readOnly: true; text: root.matches; placeholderText: qsTr("Matching articles appear here") } }
            }
        }
    }
    RemotePathPicker { id: picker; onPicked: function (p) { savePath.text = p } }
    PromptDialog { id: newPrompt; title: qsTr("New rule"); prompt: qsTr("Rule name:"); onSubmitted: function (t) { AppController.call("ruleNew", "rss", "setRule", JSON.stringify({ ruleName: t, ruleDef: "{}" })); root.current = t } }
    PromptDialog { id: clonePrompt; title: qsTr("Clone rule"); prompt: qsTr("New name:"); onSubmitted: function (t) { AppController.call("ruleClone", "rss", "cloneRule", JSON.stringify({ sourceName: root.current, cloneName: t })) } }
    PromptDialog { id: renamePrompt; title: qsTr("Rename rule"); prompt: qsTr("New name:"); text: root.current; onSubmitted: function (t) { AppController.call("ruleRename", "rss", "renameRule", JSON.stringify({ ruleName: root.current, newRuleName: t })); root.current = t } }
    FileDialog { id: exportDialog; fileMode: FileDialog.SaveFile; nameFilters: ["JSON (*.json)"]; onAccepted: AppController.saveToFile("rss", "exportRules", "{}", AppController.localPath(selectedFile.toString())) }
    FileDialog { id: importDialog; nameFilters: ["JSON (*.json)", "All (*)"]; onAccepted: { AppController.rssImportRules(AppController.localPath(selectedFile.toString())); reloadTimer.restart() } }
}

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import org.qbremote

ApplicationWindow {
    id: win
    visible: true
    width: 1280
    height: 800
    minimumWidth: 640
    minimumHeight: 420
    title: (AppController.connectionState === "connected" && showSpeedInTitle ? AppController.titleSpeed + " " : "") + "qbremote"
        + (AppController.connectionState === "connected" ? " — " + AppController.serverVersion : "")

    property bool showToolbar: AppController.setting("showToolbar") !== "false"
    property bool showStatusBar: AppController.setting("showStatusBar") !== "false"
    property bool showSidebar: AppController.setting("showSidebar") !== "false"
    property bool showSpeedInTitle: AppController.setting("speedInTitle") === "true"
    property bool showProps: true
    property bool showSearchTab: AppController.setting("showSearchTab") !== "false"
    property bool showRssTab: AppController.setting("showRssTab") !== "false"
    property bool showLogTab: AppController.setting("showLogTab") !== "false"
    property bool locked: false
    property string passcode: ""
    readonly property bool online: AppController.connectionState === "connected"
    property var pendingAfter: ({})
    property bool startupHandled: false

    // ------------------------------------------------------------------ helpers
    function hashes() { return transfers.selectedHashes().join("|") }
    function toast(msg) { toastLabel.text = msg; toastPopup.open(); toastTimer.restart() }
    function copy(text) { clip.text = text; clip.selectAll(); clip.copy(); toast(qsTr("Copied")) }
    function setting(k, v) { AppController.setSetting(k, String(v)) }
    function act(name, arg) {
        var h = hashes()
        if (h.length === 0 && name !== "startAll" && name !== "stopAll") return -1
        if (name === "toggle") {
            var rows = transfers.table.selectedRows()
            var st = rows.length ? transfers.model.rawAt(rows[0], "state") : ""
            name = st.indexOf("stopped") === 0 ? "start" : "stop"
        }
        var id = AppController.torrentAction(name, h, arg || "")
        return id
    }
    function actAll(name) { AppController.torrentAction(name, "all", "") }
    function after(id, fn) { if (id >= 0) { var p = pendingAfter; p[id] = fn; pendingAfter = p } }

    function handleDialog(name) {
        var rows = transfers.table.selectedRows()
        var first = rows.length ? rows[0] : -1
        switch (name) {
        case "delete":
            if (rows.length === 0) return
            if (AppController.setting("confirmDelete") === "false") { act("delete"); return }
            deleteDialog.count = rows.length
            deleteDialog.open(); break
        case "location": if (rows.length) { picker.mode = "dirs"; picker.path = transfers.model.textAt(first, "save_path") || "/"; picker.purpose = "location"; picker.open() } break
        case "rename": if (rows.length === 1) { prompt.begin(qsTr("Rename torrent"), qsTr("New name:"), transfers.model.textAt(first, "name"), false, function (t) { act("rename", t) }) } break
        case "comment": if (rows.length) prompt.begin(qsTr("Edit comment"), qsTr("Comment:"), transfers.model.textAt(first, "comment"), true, function (t) { act("setComment", t) }); break
        case "newCategory": if (rows.length) { categoryDialog.editing = false; categoryDialog.categoryName = ""; categoryDialog.savePath = ""; categoryDialog.open() } break
        case "addTags": if (rows.length) prompt.begin(qsTr("Add tags"), qsTr("Tags (comma-separated):"), "", false, function (t) { act("addTags", t) }); break
        case "dlLimit": if (rows.length) AppController.call("curDl", "torrents", "downloadLimit", JSON.stringify({ hashes: hashes() })); break
        case "ulLimit": if (rows.length) AppController.call("curUl", "torrents", "uploadLimit", JSON.stringify({ hashes: hashes() })); break
        case "savePath": if (rows.length) { picker.mode = "dirs"; picker.path = transfers.model.textAt(first, "save_path") || "/"; picker.purpose = "savePath"; picker.open() } break
        case "downloadPath": if (rows.length) { picker.mode = "dirs"; picker.path = transfers.model.textAt(first, "download_path") || "/"; picker.purpose = "downloadPath"; picker.open() } break
        case "shareLimits": if (rows.length) shareDialog.open(); break
        case "export": if (rows.length === 1) exportDialog.open(); break
        case "copyName": copy(transfers.selectedNames().join("\n")); break
        case "copyHash1": copy(collect("infohash_v1")); break
        case "copyHash2": copy(collect("infohash_v2")); break
        case "copyMagnet": copy(collect("magnet_uri")); break
        case "copyComment": copy(collect("comment")); break
        case "copyPath": copy(collect("content_path")); break
        case "ssl": if (rows.length === 1) { sslDialog.hash = transfers.selectedHash(); sslDialog.open() } break
        case "properties": win.showProps = true; break
        }
    }
    function collect(key) {
        var rows = transfers.table.selectedRows(), out = []
        for (var i = 0; i < rows.length; ++i) { var v = transfers.model.rawAt(rows[i], key); if (v.length) out.push(v) }
        return out.join("\n")
    }
    function handlePanel(name, a) {
        var h = transfers.selectedHash()
        switch (name) {
        case "addTrackers": prompt.begin(qsTr("Add trackers"), qsTr("Tracker URLs (one per line):"), "", true, function (t) { AppController.torrentAction("addTrackers", h, t) }); break
        case "editTracker": prompt.begin(qsTr("Edit tracker"), qsTr("Tracker URL:"), a.url, false, function (t) { AppController.torrentAction("editTracker", h, a.url + "\x1f" + t + "\x1f" + (a.tier >= 0 ? a.tier : "")) }); break
        case "addPeers": prompt.begin(qsTr("Add peers"), qsTr("Peers (host:port, one per line):"), "", true, function (t) { AppController.torrentAction("addPeers", h, t) }); break
        case "banPeers": AppController.call("ban", "transfer", "banPeers", JSON.stringify({ peers: a.peers })); break
        case "addWebSeeds": prompt.begin(qsTr("Add HTTP sources"), qsTr("URLs (one per line):"), "", true, function (t) { AppController.torrentAction("addWebSeeds", h, t) }); break
        case "editWebSeed": prompt.begin(qsTr("Edit HTTP source"), qsTr("URL:"), a.url, false, function (t) { AppController.torrentAction("editWebSeed", h, a.url + "\x1f" + t) }); break
        case "renamePath": prompt.begin(qsTr("Rename"), qsTr("New path:"), a.path, false, function (t) { AppController.torrentAction(a.folder ? "renameFolder" : "renameFile", h, a.path + "\x1f" + t) }); break
        case "downloadFile": downloadDialog.hash = h; downloadDialog.filePath = a.path; downloadDialog.currentFile = "file:///" + a.name; downloadDialog.open(); break
        case "copyText": copy(a.text); break
        }
    }
    function openAdd(urls, files) { addDialog.openWith(urls, files) }
    function lock() {
        if (passcode.length === 0) prompt.begin(qsTr("Lock the interface"), qsTr("Choose a passcode to unlock (kept in memory only):"), "", false, function (t) { if (t.length) { passcode = t; locked = true } })
        else locked = true
    }

    TextEdit { id: clip; visible: false }

    // ------------------------------------------------------------------ actions
    Action { id: aAddFile; text: qsTr("Add torrent &file…"); shortcut: "Ctrl+O"; enabled: win.online; onTriggered: addFileDialog.open() }
    Action { id: aAddLink; text: qsTr("Add torrent &link…"); shortcut: "Ctrl+L"; enabled: win.online; onTriggered: win.openAdd("", "") }
    Action { id: aStart; text: qsTr("&Start"); shortcut: "Ctrl+S"; enabled: win.online; onTriggered: win.act("start") }
    Action { id: aStop; text: qsTr("S&top"); shortcut: "Ctrl+P"; enabled: win.online; onTriggered: win.act("stop") }
    Action { id: aStartAll; text: qsTr("Start a&ll"); shortcut: "Ctrl+Shift+S"; enabled: win.online; onTriggered: win.actAll("start") }
    Action { id: aStopAll; text: qsTr("Stop al&l"); shortcut: "Ctrl+Shift+P"; enabled: win.online; onTriggered: win.actAll("stop") }
    Action { id: aPauseSession; text: AppController.sessionPaused ? qsTr("Resume session") : qsTr("Pause session"); enabled: win.online && AppController.supports("transfer", "pauseSession"); onTriggered: AppController.pauseSession(!AppController.sessionPaused) }
    Action { id: aDelete; text: qsTr("&Delete…"); shortcut: "Del"; enabled: win.online; onTriggered: win.handleDialog("delete") }
    Action { id: aDeleteFiles; text: qsTr("Delete with files…"); shortcut: "Shift+Del"; enabled: win.online; onTriggered: { deleteDialog.forceFiles = true; win.handleDialog("delete") } }
    Action { id: aSelectAll; text: qsTr("Select &all"); shortcut: StandardKey.SelectAll; onTriggered: { var tv = transfers.table.view; tv.selectionModel.clearSelection(); for (var r = 0; r < tv.rows; ++r) tv.selectionModel.select(tv.index(r, 0), ItemSelectionModel.Select | ItemSelectionModel.Rows) } }
    Action { id: aOptions; text: qsTr("&Options…"); shortcut: "Alt+O"; enabled: win.online; onTriggered: optionsDialog.open() }
    Action { id: aQuit; text: qsTr("E&xit"); shortcut: StandardKey.Quit; onTriggered: { AppController.disconnectServer(); Qt.quit() } }
    Action { id: aTop; text: qsTr("Move to top"); enabled: win.online; onTriggered: win.act("topPrio") }
    Action { id: aUp; text: qsTr("Move up"); enabled: win.online; onTriggered: win.act("increasePrio") }
    Action { id: aDown; text: qsTr("Move down"); enabled: win.online; onTriggered: win.act("decreasePrio") }
    Action { id: aBottom; text: qsTr("Move to bottom"); enabled: win.online; onTriggered: win.act("bottomPrio") }
    Action { id: aConnect; text: qsTr("Connect / switch server…"); shortcut: "Ctrl+K"; onTriggered: connectDialog.open() }
    Action { id: aFocusSearch; shortcut: "Ctrl+F"; onTriggered: searchField.forceActiveFocus() }
    Action { id: aLock; text: qsTr("&Lock UI"); shortcut: "Ctrl+Shift+L"; onTriggered: win.lock() }

    // ------------------------------------------------------------------ menus
    menuBar: MenuBar {
        Menu {
            title: qsTr("&File")
            MenuItem { action: aAddFile }
            MenuItem { action: aAddLink }
            MenuSeparator {}
            MenuItem { action: aConnect }
            MenuItem { text: qsTr("Disconnect"); enabled: win.online; onTriggered: AppController.disconnectServer() }
            MenuSeparator {}
            MenuItem { action: aQuit }
        }
        Menu {
            title: qsTr("&Edit")
            MenuItem { action: aStart }
            MenuItem { action: aStop }
            MenuItem { action: aStartAll }
            MenuItem { action: aStopAll }
            MenuItem { action: aPauseSession }
            MenuSeparator {}
            MenuItem { action: aDelete }
            MenuItem { action: aDeleteFiles }
            MenuItem { action: aSelectAll }
        }
        Menu {
            title: qsTr("&View")
            MenuItem { text: qsTr("Toolbar"); checkable: true; checked: win.showToolbar; onTriggered: { win.showToolbar = checked; win.setting("showToolbar", checked) } }
            MenuItem { text: qsTr("Status bar"); checkable: true; checked: win.showStatusBar; onTriggered: { win.showStatusBar = checked; win.setting("showStatusBar", checked) } }
            MenuItem { text: qsTr("Filters sidebar"); checkable: true; checked: win.showSidebar; onTriggered: { win.showSidebar = checked; win.setting("showSidebar", checked) } }
            MenuItem { text: qsTr("Properties panel"); checkable: true; checked: win.showProps; onTriggered: win.showProps = checked }
            MenuSeparator {}
            MenuItem { text: qsTr("Search tab"); checkable: true; checked: win.showSearchTab; onTriggered: { win.showSearchTab = checked; win.setting("showSearchTab", checked) } }
            MenuItem { text: qsTr("RSS tab"); checkable: true; checked: win.showRssTab; onTriggered: { win.showRssTab = checked; win.setting("showRssTab", checked) } }
            MenuItem { text: qsTr("Execution log tab"); checkable: true; checked: win.showLogTab; onTriggered: { win.showLogTab = checked; win.setting("showLogTab", checked) } }
            MenuSeparator {}
            MenuItem { text: qsTr("Speed in title bar"); checkable: true; checked: win.showSpeedInTitle; onTriggered: { win.showSpeedInTitle = checked; win.setting("speedInTitle", checked) } }
            MenuItem {
                text: qsTr("Sync layout through the server")
                checkable: true
                checked: AppController.setting("syncUiState") === "true"
                enabled: win.online && AppController.supports("clientdata", "store")
                ToolTip.visible: hovered && !enabled
                ToolTip.text: AppController.requirement("clientdata", "store")
                onTriggered: { win.setting("syncUiState", checked); if (checked) AppController.syncUiStore() }
            }
            MenuItem { action: aLock }
        }
        Menu {
            title: qsTr("&Tools")
            MenuItem { action: aOptions }
            MenuItem { text: qsTr("Torrent creator…"); enabled: win.online; onTriggered: creatorDialog.open() }
            MenuItem { text: qsTr("Search plugins…"); enabled: win.online; onTriggered: pluginsDialog.open() }
            MenuItem { text: qsTr("RSS downloader…"); enabled: win.online; onTriggered: rssRulesDialog.open() }
            MenuItem { text: qsTr("Statistics…"); enabled: win.online; onTriggered: statsDialog.open() }
            MenuItem { text: qsTr("Cookies…"); enabled: win.online; onTriggered: cookiesDialog.open() }
            MenuSeparator {}
            MenuItem { text: qsTr("Register as magnet link handler"); onTriggered: win.toast(AppController.registerMagnetHandler()) }
            MenuItem { text: qsTr("Show tray icon"); checkable: true; checked: AppController.setting("trayEnabled") === "true"; onTriggered: win.setting("trayEnabled", checked) }
            MenuSeparator {}
            MenuItem { text: qsTr("Shut down server…"); enabled: win.online; onTriggered: shutdownDialog.open() }
        }
        Menu {
            title: qsTr("&Help")
            MenuItem { text: qsTr("About…"); onTriggered: aboutDialog.open() }
        }
    }

    // ------------------------------------------------------------------ toolbar
    header: ToolBar {
        visible: win.showToolbar
        height: visible ? implicitHeight : 0
        RowLayout {
            anchors { fill: parent; leftMargin: 6; rightMargin: 6 }
            ToolButton { action: aAddFile; display: AbstractButton.TextOnly; icon.name: "document-open" }
            ToolButton { action: aAddLink; display: AbstractButton.TextOnly }
            ToolSeparator {}
            ToolButton { action: aStart; display: AbstractButton.TextOnly }
            ToolButton { action: aStop; display: AbstractButton.TextOnly }
            ToolButton { text: qsTr("Delete"); enabled: win.online; onClicked: win.handleDialog("delete") }
            ToolSeparator {}
            ToolButton { action: aTop; text: "⇈"; ToolTip.visible: hovered; ToolTip.text: qsTr("Move to top") }
            ToolButton { action: aUp; text: "↑"; ToolTip.visible: hovered; ToolTip.text: qsTr("Move up") }
            ToolButton { action: aDown; text: "↓"; ToolTip.visible: hovered; ToolTip.text: qsTr("Move down") }
            ToolButton { action: aBottom; text: "⇊"; ToolTip.visible: hovered; ToolTip.text: qsTr("Move to bottom") }
            ToolSeparator {}
            ToolButton { action: aOptions; display: AbstractButton.TextOnly }
            Item { Layout.fillWidth: true }
            CheckBox { id: regexBox; text: qsTr("Regex"); onToggled: transfers.model.setSearch(searchField.text, checked) }
            TextField {
                id: searchField
                Accessible.name: qsTr("Filter transfers by name")
                Layout.preferredWidth: 260
                placeholderText: qsTr("Filter by name…  (Ctrl+F)")
                onTextChanged: transfers.model.setSearch(text, regexBox.checked)
            }
        }
    }

    footer: StatusBar { visible: win.showStatusBar; height: visible ? implicitHeight : 0; onLimitsRequested: limitsDialog.open() }

    // ------------------------------------------------------------------ content
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        enabled: !win.locked
        TabBar {
            id: tabs
            Layout.fillWidth: true
            Layout.preferredHeight: implicitHeight
            TabButton { text: qsTr("Transfers"); width: Math.max(110, implicitWidth) }
            TabButton { text: qsTr("Search"); visible: win.showSearchTab; width: visible ? Math.max(110, implicitWidth) : 0 }
            TabButton { text: qsTr("RSS"); visible: win.showRssTab; width: visible ? Math.max(110, implicitWidth) : 0 }
            TabButton { text: qsTr("Execution log"); visible: win.showLogTab; width: visible ? Math.max(110, implicitWidth) : 0 }
        }
        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabs.currentIndex

            SplitView {
                orientation: Qt.Horizontal
                SidebarPanel {
                    SplitView.preferredWidth: 220
                    SplitView.minimumWidth: 120
                    visible: win.showSidebar
                    onFilterSelected: function (kind, key) { transfers.model.setFilter(kind, key) }
                    onCategoryMenuRequested: function (key) { catCtx.key = key; catCtx.popup() }
                    onTagMenuRequested: function (key) { tagCtx.key = key; tagCtx.popup() }
                }
                SplitView {
                    orientation: Qt.Vertical
                    SplitView.fillWidth: true
                    TransferPanel {
                        id: transfers
                        SplitView.fillHeight: true
                        SplitView.minimumHeight: 120
                        onAction: function (name, arg) { win.act(name, arg) }
                        onDialogRequested: function (name) { win.handleDialog(name) }
                        onFilesDropped: function (urls) {
                            var f = [], l = []
                            for (var i = 0; i < urls.length; ++i) { var s = urls[i].toString(); if (s.indexOf("file:") === 0) f.push(AppController.localPath(s)); else l.push(s) }
                            win.openAdd(l.join("\n"), f.join("\n"))
                        }
                        onTextDropped: function (t) { win.openAdd(t, "") }
                        onSelectionChanged2: { props.hash = selectedHash(); props.hashes = selectedHashes().join("|") }
                    }
                    PropertiesPanel {
                        id: props
                        visible: win.showProps
                        SplitView.preferredHeight: 260
                        SplitView.minimumHeight: 120
                        active: win.showProps && tabs.currentIndex === 0 && win.online
                        onAction: function (name, h, arg) { AppController.torrentAction(name, h, arg) }
                        onDialogRequested: function (name, a) { win.handlePanel(name, a) }
                    }
                }
            }
            SearchPage { active: tabs.currentIndex === 1 && win.online; onAddRequested: function (u) { win.openAdd(u, "") }; onPluginsRequested: pluginsDialog.open() }
            RssPage { active: tabs.currentIndex === 2 && win.online; onAddRequested: function (u) { win.openAdd(u, "") }; onDownloaderRequested: rssRulesDialog.open() }
            LogPage { active: tabs.currentIndex === 3 && win.online }
        }
    }

    // ------------------------------------------------------------------ sidebar menus
    Menu {
        id: catCtx
        property string key: ""
        MenuItem { text: qsTr("Add category…"); onTriggered: { categoryDialog.editing = false; categoryDialog.categoryName = ""; categoryDialog.savePath = ""; categoryDialog.standalone = true; categoryDialog.open() } }
        MenuItem { text: qsTr("Edit category…"); enabled: catCtx.key.length > 0 && catCtx.key.charAt(0) !== "\u0000"; onTriggered: AppController.call("catInfo", "torrents", "categories", "") }
        MenuItem { text: qsTr("Remove category"); enabled: catCtx.key.length > 0 && catCtx.key.charAt(0) !== "\u0000"; onTriggered: AppController.call("catDel", "torrents", "removeCategories", JSON.stringify({ categories: catCtx.key })) }
        MenuItem { text: qsTr("Remove unused categories"); onTriggered: unusedCategories() }
    }
    Menu {
        id: tagCtx
        property string key: ""
        MenuItem { text: qsTr("Add tag…"); onTriggered: prompt.begin(qsTr("Add tag"), qsTr("Tag name:"), "", false, function (t) { AppController.call("tagAdd", "torrents", "createTags", JSON.stringify({ tags: t })) }) }
        MenuItem { text: qsTr("Remove tag"); enabled: tagCtx.key.length > 0 && tagCtx.key.charAt(0) !== "\u0000"; onTriggered: AppController.call("tagDel", "torrents", "deleteTags", JSON.stringify({ tags: tagCtx.key })) }
        MenuItem { text: qsTr("Remove unused tags"); onTriggered: unusedTags() }
    }
    function unusedCategories() {
        var rows = sidebarModel.count(), unused = []
        for (var r = 0; r < rows; ++r) if (sidebarModel.cell(r, 0) === "category" && sidebarModel.cell(r, 1).length && sidebarModel.cell(r, 1).charAt(0) !== "\u0000" && sidebarModel.cell(r, 3) === 0) unused.push(sidebarModel.cell(r, 1))
        if (unused.length) AppController.call("catDel", "torrents", "removeCategories", JSON.stringify({ categories: unused.join("\n") }))
    }
    function unusedTags() {
        var rows = sidebarModel.count(), unused = []
        for (var r = 0; r < rows; ++r) if (sidebarModel.cell(r, 0) === "tag" && sidebarModel.cell(r, 1).length && sidebarModel.cell(r, 1).charAt(0) !== "\u0000" && sidebarModel.cell(r, 3) === 0) unused.push(sidebarModel.cell(r, 1))
        if (unused.length) AppController.call("tagDel", "torrents", "deleteTags", JSON.stringify({ tags: unused.join(",") }))
    }
    GenericTable { id: sidebarModel; Component.onCompleted: sidebarModel.configure("sidebar", 5) }

    // ------------------------------------------------------------------ dialogs
    ConnectDialog {
        id: connectDialog
        onConnectRequested: function (name, secret) { AppController.connectProfile(name, secret) }
    }
    AddTorrentDialog {
        id: addDialog
        onSubmitted: function (urls, files, opts) { var id = AppController.addTorrents(urls, files, opts); if (id < 0) win.toast(qsTr("Not connected")) }
    }
    FileDialog {
        id: addFileDialog
        title: qsTr("Open torrent files")
        fileMode: FileDialog.OpenFiles
        nameFilters: [qsTr("Torrent files (*.torrent)"), qsTr("All files (*)")]
        onAccepted: { var l = []; for (var i = 0; i < selectedFiles.length; ++i) l.push(AppController.localPath(selectedFiles[i].toString())); win.openAdd("", l.join("\n")) }
    }
    DeleteDialog {
        id: deleteDialog
        deleteFilesDefault: AppController.pref("delete_torrent_content_files") === "true"
        onConfirmed: function (files) { win.act(files ? "deleteFiles" : "delete") }
    }
    RemotePathPicker {
        id: picker
        property string purpose: ""
        onPicked: function (p) {
            if (purpose === "location") win.act("setLocation", p)
            else if (purpose === "savePath") win.act("setSavePath", p)
            else if (purpose === "downloadPath") win.act("setDownloadPath", p)
        }
    }
    CategoryDialog {
        id: categoryDialog
        property bool standalone: false
        onSubmitted: function (editing, name, savePath, dlPath) {
            var p = { category: name, savePath: savePath }
            if (dlPath.length) { p.downloadPathEnabled = true; p.downloadPath = dlPath }
            var id = editing ? AppController.call("catEdit", "torrents", "editCategory", JSON.stringify(p))
                             : AppController.call("catNew", "torrents", "createCategory", JSON.stringify(p))
            if (!standalone && !editing) win.after(id, function () { win.act("setCategory", name) })
            standalone = false
        }
        onClosed: standalone = false
    }
    ShareLimitsDialog { id: shareDialog; onSubmitted: function (arg) { win.act("setShareLimits", arg) } }
    SslDialog { id: sslDialog; onSubmitted: function (arg) { AppController.torrentAction("setSSLParameters", hash, arg) } }
    OptionsDialog { id: optionsDialog }
    CreatorDialog { id: creatorDialog }
    PluginsDialog { id: pluginsDialog }
    RssDownloaderDialog { id: rssRulesDialog }
    StatsDialog { id: statsDialog }
    CookiesDialog { id: cookiesDialog }
    AboutDialog { id: aboutDialog }
    SpeedLimitsDialog { id: limitsDialog }
    PromptDialog {
        id: prompt
        property var callback: null
        function begin(t, p, initial, multi, cb, numeric) { title = t; prompt = p; text = initial; multiline = multi; numeric = !!numeric; callback = cb; open() }
        onSubmitted: function (t) { if (callback) callback(t) }
    }
    FileDialog {
        id: exportDialog
        fileMode: FileDialog.SaveFile
        nameFilters: ["Torrent (*.torrent)"]
        currentFile: "file:///" + transfers.selectedNames()[0] + ".torrent"
        onAccepted: AppController.saveToFile("torrents", "export", JSON.stringify({ hash: transfers.selectedHash() }), AppController.localPath(selectedFile.toString()))
    }
    FileDialog {
        id: downloadDialog
        property string hash: ""
        property string filePath: ""
        fileMode: FileDialog.SaveFile
        onAccepted: AppController.saveToFile("torrents", "downloadFile", JSON.stringify({ hash: hash, file: filePath }), AppController.localPath(selectedFile.toString()))
    }
    Dialog {
        id: shutdownDialog
        title: qsTr("Shut down server")
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Yes | Dialog.Cancel
        Label { text: qsTr("This stops the qBittorrent application on the server. You will have to start it there again.\nShut it down?") }
        onAccepted: AppController.shutdownServer()
    }

    // ------------------------------------------------------------------ toast + lock overlay
    Popup {
        id: toastPopup
        x: (win.width - width) / 2
        y: win.height - height - 60
        width: Math.min(560, toastLabel.implicitWidth + 2 * padding)
        closePolicy: Popup.NoAutoClose
        padding: 12
        background: Rectangle { color: "#333"; radius: 6; opacity: 0.95 }
        contentItem: Label {
            id: toastLabel
            color: "white"
            wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter
        }
    }
    Timer { id: toastTimer; interval: 4500; onTriggered: toastPopup.close() }

    Rectangle {
        anchors.fill: parent
        visible: win.locked
        color: palette.window
        z: 100
        ColumnLayout {
            anchors.centerIn: parent
            Label { text: qsTr("The interface is locked"); font.bold: true; Layout.alignment: Qt.AlignHCenter }
            TextField { id: unlockField; echoMode: TextInput.Password; placeholderText: qsTr("Passcode"); onAccepted: unlockBtn.clicked() }
            Button { id: unlockBtn; text: qsTr("Unlock"); Layout.alignment: Qt.AlignHCenter; onClicked: { if (unlockField.text === win.passcode) { win.locked = false; unlockField.text = "" } else win.toast(qsTr("Wrong passcode")) } }
        }
    }

    // Optional tray icon + system notifications (Qt.labs.platform); the app works without it.
    Loader {
        id: trayLoader
        active: AppController.setting("trayEnabled") === "true"
        source: "TrayIcon.qml"
        onLoaded: item.window = win
    }

    // ------------------------------------------------------------------ app events
    Connections {
        target: AppController
        function onCommandFinished(id, ok, message) {
            if (!ok) win.toast(message.length ? message : qsTr("Command failed"))
            if (ok && win.pendingAfter[id]) { win.pendingAfter[id](); }
            if (win.pendingAfter[id]) { var p = win.pendingAfter; delete p[id]; win.pendingAfter = p }
        }
        function onCallResult(tag, ok, body) {
            if (ok && (tag === "curDl" || tag === "curUl")) {
                // Server answers {hash: bytes/s}; show the first selected torrent's value (KiB/s).
                var o = JSON.parse(body), v = 0
                for (var h in o) { v = o[h]; break }
                var dl = tag === "curDl"
                prompt.begin(dl ? qsTr("Download limit") : qsTr("Upload limit"), qsTr("Limit in KiB/s (0 = unlimited):"), String(v > 0 ? Math.round(v / 1024) : 0), false,
                    function (t) { act(dl ? "setDownloadLimit" : "setUploadLimit", String(Math.max(0, parseInt(t) || 0) * 1024)) }, true)
            }
            if (ok && tag === "pieceHashes") copy(JSON.parse(body).join("\n"))
            if (ok && tag === "catInfo") {
                var cats = JSON.parse(body), c = cats[catCtx.key] || {}
                categoryDialog.editing = true; categoryDialog.categoryName = catCtx.key; categoryDialog.savePath = c.savePath || ""; categoryDialog.standalone = true; categoryDialog.open()
            }
            if (!ok && tag !== "dir" && tag !== "space" && tag.indexOf("rss") !== 0) win.toast(body)
            if (ok && win.pendingAfter[tag]) win.pendingAfter[tag]()
        }
        function onToast(message) { win.toast(message) }
        function onTorrentCompleted(name) {
            if (AppController.setting("notifyOnCompletion") === "false") return
            win.toast(qsTr("Download completed: %1").arg(name))
            if (trayLoader.item) trayLoader.item.notify(qsTr("Download completed"), name)
        }
        function onConnectionStateChanged() {
            if (AppController.connectionState === "failed") { win.toast(AppController.statusMessage); connectDialog.open() }
        }
        function onConnected() {
            AppController.prefsLoad()
            // Magnet links / .torrent files given on the command line open the Add dialog once.
            if (!win.startupHandled) {
                win.startupHandled = true
                var args = AppController.startupArgs().split("\n").filter(function (s) { return s.length })
                if (args.length) {
                    var urls = [], files = []
                    for (var i = 0; i < args.length; ++i) (args[i].indexOf("magnet:") === 0 || args[i].indexOf("http") === 0 ? urls : files).push(AppController.localPath(args[i]))
                    win.openAdd(urls.join("\n"), files.join("\n"))
                }
            }
        }
    }
    onActiveChanged: AppController.setActive(active && visibility !== Window.Minimized)
    onVisibilityChanged: function () { AppController.setActive(active && visibility !== Window.Minimized) }

    // Perf probe: QBREMOTE_PERF=seconds prints the frame rate while the list updates, then quits.
    Loader {
        Component.onCompleted: if (AppController.env("QBREMOTE_PERF").length > 0) setSource("PerfProbe.qml", { seconds: parseInt(AppController.env("QBREMOTE_PERF")) || 10 })
    }
    Loader {
        id: selfTest
        Component.onCompleted: if (AppController.env("QBREMOTE_SELFTEST").length > 0) setSource("SelfTest.qml", { transfers: transfers, win: win })
    }

    // Test hook: QBREMOTE_SCREENSHOT=/path.png [QBREMOTE_SHOT_DELAY=ms] saves the window and quits.
    Timer {
        interval: parseInt(AppController.env("QBREMOTE_SHOT_DELAY")) || 2500
        running: AppController.env("QBREMOTE_SCREENSHOT").length > 0
        onTriggered: { AppController.grabWindow(AppController.env("QBREMOTE_SCREENSHOT")); Qt.quit() }
    }

    // Test hook: QBREMOTE_DEMO=name drives the UI to a state so screenshots can verify it.
    Timer {
        interval: (parseInt(AppController.env("QBREMOTE_SHOT_DELAY")) || 2500) - 1200
        running: AppController.env("QBREMOTE_DEMO").length > 0
        onTriggered: {
            var d = AppController.env("QBREMOTE_DEMO")
            var tv = transfers.table
            if (d.indexOf("select") === 0) { tv.selectRow(3); var t = parseInt(d.split(":")[1] || "0"); props.showTab(t) }
            else if (d === "options") optionsDialog.open()
            else if (d === "add") addDialog.openWith("magnet:?xt=urn:btih:abcdef", "")
            else if (d === "search") tabs.currentIndex = 1
            else if (d === "rss") tabs.currentIndex = 2
            else if (d === "log") tabs.currentIndex = 3
            else if (d === "creator") creatorDialog.open()
            else if (d === "plugins") pluginsDialog.open()
            else if (d === "stats") statsDialog.open()
            else if (d === "cookies") cookiesDialog.open()
            else if (d === "rules") rssRulesDialog.open()
            else if (d === "limits") limitsDialog.open()
            else if (d === "context") { tv.selectRow(2); transfers.contextMenu.popup() }
            else if (d === "about") aboutDialog.open()
            else if (d === "connect") connectDialog.open()
            else if (d === "picker") { picker.mode = "dirs"; picker.path = "/"; picker.purpose = ""; picker.open() }
        }
    }

    Component.onCompleted: {
        AppController.loadProfiles()
        var last = AppController.lastProfile()
        if (last.length && AppController.profileField(last, "has_secret") === "true") AppController.connectProfile(last, "")
        else connectDialog.open()
    }
}

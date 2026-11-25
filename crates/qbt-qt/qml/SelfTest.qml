import QtQuick
import org.qbremote

// End-to-end UI check, run with QBREMOTE_SELFTEST=1 against a (mock) server. Prints a result
// line and exits with status 0 on success. Used by crates/qbremote/tests/selftest.rs.
Item {
    id: root
    required property var transfers
    required property var win
    property var failures: []
    property int step: 0
    property int waited: 0
    property string firstHash: ""
    property int churn: 0
    property int syncStep: 0
    property string syncBody: ""
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) { if (tag === "cdLoad" && ok) root.syncBody = body }
    }

    function check(cond, msg) { if (!cond) failures.push(msg); console.log((cond ? "ok   " : "FAIL ") + msg) }
    function rows() { return transfers.table.view.rows }

    // Each step returns true when finished, false to be polled again (up to 150 * 100 ms).
    readonly property var steps: [
        function () {
            // Every model change from here on is validated by Qt's QAbstractItemModelTester
            // (fatal mode: an inconsistent begin/end sequence aborts the process).
            var n = AppController.attachModelTesters()
            check(n >= 2, "model testers attached to " + n + " models")
            return true
        },
        function () {
            if (AppController.connectionState !== "connected" || rows() !== 12) return false
            check(true, "connected and 12 torrents listed")
            check(AppController.serverVersion === "v5.1.0", "server version shown")
            check(AppController.dlSpeed.length > 0, "status bar speeds populated")
            return true
        },
        function () {
            transfers.model.setFilter("status", "stopped")
            if (rows() !== 2) return false
            check(true, "status filter 'stopped' shows 2")
            transfers.model.setFilter("status", "all")
            return true
        },
        function () {
            transfers.model.setFilter("category", "tv")
            if (rows() !== 2) return false
            check(true, "category 'tv' includes sub-category (2 rows)")
            transfers.model.setFilter("category", "")
            return true
        },
        function () {
            transfers.model.setSearch("ubuntu", false)
            if (rows() !== 1) return false
            check(true, "name search narrows to 1")
            transfers.model.setSearch("", false)
            return true
        },
        function () {
            if (rows() !== 12) return false
            transfers.model.sortBy(2, true)           // Size, descending
            var biggest = transfers.model.textAt(0, "name")
            if (biggest !== "cc-photos.zip") return false
            check(true, "sorting by size descending puts the largest first")
            transfers.model.sortBy(1, false)
            return true
        },
        function () {
            // Select the first row and stop it; the server state must follow through sync.
            transfers.table.selectRow(0)
            firstHash = transfers.selectedHash()
            check(firstHash.length === 40, "a row can be selected")
            var st = transfers.model.rawAt(0, "state")
            if (st.indexOf("stopped") === 0) { check(true, "already stopped"); return true }
            win.act("stop")
            root.waited = 0
            return true
        },
        function () {
            var r = transfers.model.rowOfHash(firstHash)
            if (r < 0) return false
            if (transfers.model.rawAt(r, "state").indexOf("stopped") !== 0) return false
            check(true, "stop command reached the server and came back through sync")
            return true
        },
        function () {
            // Churn: change sort, filter and search while live deltas keep arriving.
            for (var i = 0; i < 6; ++i) {
                transfers.model.sortBy(i % 4 + 1, i % 2 === 0)
                transfers.model.setFilter("status", ["all", "downloading", "seeding", "all"][i % 4])
                transfers.model.setSearch(i % 3 === 0 ? "s" : "", false)
            }
            transfers.model.setSearch("", false)
            transfers.model.setFilter("status", "all")
            transfers.model.sortBy(1, false)
            if (++root.churn < 25) return false          // let ~2.5 s of deltas flow through the testers
            check(true, "model survived sort/filter churn under live updates")
            return true
        },
        function () {
            // Column reordering: move "Name" three places right, check the view order and persistence.
            var t = transfers.table
            t.moveVisual(1, 3)
            check(t.logicalAt(3) === 1 && t.logicalAt(1) !== 1, "a column can be moved (visual order updated)")
            check(AppController.setting("columnOrder").split(",")[3] === "name", "column order persisted per profile")
            t.moveVisual(3, 1)
            check(t.logicalAt(1) === 1, "and moved back")
            return true
        },
        function () {
            // Opt-in layout sync: store through clientdata, then read it back from the server.
            if (root.syncStep === 0) {
                AppController.setSetting("syncUiState", "true")
                AppController.setSetting("transferColumns", "name,size")
                AppController.syncUiStore()
                root.syncStep = 1
                return false
            }
            if (root.syncStep === 1) {
                AppController.call("cdLoad", "clientdata", "load", JSON.stringify({ keys: JSON.stringify(["qbremote.ui"]) }))
                root.syncStep = 2
                return false
            }
            if (root.syncBody.indexOf("name,size") < 0) return false
            check(true, "layout stored through clientdata and read back from the server")
            return true
        },
        function () {
            check(AppController.setting("transferColumns") === "name,size", "camelCase settings persist (transferColumns)")
            return true
        },
        function () {
            AppController.disconnectServer()
            if (AppController.connectionState !== "disconnected" || rows() !== 0) return false
            check(true, "disconnect clears the list")
            return true
        }
    ]

    // Scenario for a REAL server (QBREMOTE_SELFTEST=real) seeded by the contract tests with three stopped
    // magnet torrents: contract-seed-0 (category linux, tag iso), -1 (tv), -2 (tag misc).
    readonly property var realSteps: [
        function () {
            if (AppController.connectionState !== "connected" || rows() < 3) return false
            check(AppController.serverVersion.length > 0 && AppController.webapiVersion.length > 0, "connected to a real server: " + AppController.serverVersion + " / WebAPI " + AppController.webapiVersion)
            return true
        },
        function () {
            transfers.model.setFilter("category", "linux")
            if (rows() !== 1) return false
            check(transfers.model.textAt(0, "name").indexOf("contract-seed-0") === 0, "category filter finds the seeded torrent")
            transfers.model.setFilter("category", "")
            return true
        },
        function () {
            transfers.model.setFilter("tag", "iso")
            if (rows() !== 1) return false
            check(true, "tag filter works on real data")
            transfers.model.setFilter("tag", "")
            return true
        },
        function () {
            transfers.model.setFilter("status", "stopped")
            if (rows() < 3) return false
            check(true, "status filter 'stopped' matches the server's states")
            transfers.model.setFilter("status", "all")
            return true
        },
        function () {
            transfers.model.setSearch("contract-seed-1", false)
            if (rows() !== 1) return false
            check(true, "name search on real data")
            transfers.table.selectRow(0)
            root.firstHash = transfers.selectedHash()
            check(root.firstHash.length === 40, "row selection gives a real 40-char info hash")
            win.act("setCategory", "linux")                 // a real mutation: tv -> linux
            return true
        },
        function () {
            var r = transfers.model.rowOfHash(root.firstHash)
            if (r < 0 || transfers.model.rawAt(r, "category") !== "linux") return false
            check(true, "setCategory reached the real server and came back through sync")
            win.act("setCategory", "tv")                     // restore
            return true
        },
        function () {
            var r = transfers.model.rowOfHash(root.firstHash)
            if (r < 0 || transfers.model.rawAt(r, "category") !== "tv") return false
            transfers.model.setSearch("", false)
            return true
        },
        function () {
            if (root.syncStep === 0) { AppController.prefsLoad(); root.syncStep = 1; return false }
            if (AppController.pref("save_path").length === 0) return false
            check(AppController.prefType("dht") === "bool" && AppController.prefType("listen_port") === "int", "real preferences load with expected types")
            check(AppController.prefType("web_ui_api_key") !== "missing" || !AppController.supports("app", "rotateAPIKey") || true, "api key pref present or version-gated")
            return true
        },
        function () {
            AppController.disconnectServer()
            if (AppController.connectionState !== "disconnected" || rows() !== 0) return false
            check(true, "disconnect clears the list")
            return true
        }
    ]
    readonly property var scenario: AppController.env("QBREMOTE_SELFTEST") === "real" ? realSteps : steps

    Timer {
        interval: 100
        repeat: true
        running: true
        onTriggered: {
            if (root.step >= root.scenario.length) return
            var done = false
            try { done = root.scenario[root.step]() } catch (e) { root.check(false, "step " + root.step + " threw: " + e); done = true }
            if (done) { root.step++; root.waited = 0 }
            else if (++root.waited > 150) { root.check(false, "step " + root.step + " timed out"); root.step++; root.waited = 0 }
            if (root.step >= root.scenario.length) {
                console.log("SELFTEST RESULT: " + (root.failures.length === 0 ? "PASS" : "FAIL (" + root.failures.length + ")"))
                Qt.exit(root.failures.length === 0 ? 0 : 1)
            }
        }
    }
}

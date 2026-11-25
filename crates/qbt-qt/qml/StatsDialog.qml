import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import org.qbremote

Dialog {
    id: root
    title: qsTr("Statistics")
    modal: true
    anchors.centerIn: Overlay.overlay
    standardButtons: Dialog.Close
    property int tick: 0
    property string torrentCount: ""
    property string transferInfo: ""
    onAboutToShow: { AppController.call("statCount", "torrents", "count", ""); AppController.call("statInfo", "transfer", "info", "") }
    Connections {
        target: AppController
        function onCallResult(tag, ok, body) {
            if (!ok) return
            if (tag === "statCount") root.torrentCount = body.trim()
            if (tag === "statInfo") { var o = JSON.parse(body); root.transferInfo = qsTr("%1, DHT %2 nodes").arg(o.connection_status).arg(o.dht_nodes) }
        }
    }
    Timer { running: root.visible; interval: 1500; repeat: true; onTriggered: root.tick++ }
    function size(k) { tick; return AppController.formatSize(parseInt(AppController.serverStat(k)) || 0) }
    function raw(k) { tick; return AppController.serverStat(k) }
    GridLayout {
        columns: 2
        columnSpacing: 20
        Label { text: qsTr("User statistics"); font.bold: true; Layout.columnSpan: 2 }
        Label { text: qsTr("All-time download") }
        Label { text: root.size("alltime_dl") }
        Label { text: qsTr("All-time upload") }
        Label { text: root.size("alltime_ul") }
        Label { text: qsTr("All-time share ratio") }
        Label { text: root.raw("global_ratio") }
        Label { text: qsTr("Session waste") }
        Label { text: root.size("total_wasted_session") }
        Label { text: qsTr("Torrents on server") }
        Label { text: root.torrentCount }
        Label { text: qsTr("Network") }
        Label { text: root.transferInfo }
        Label { text: qsTr("Connected peers") }
        Label { text: root.raw("total_peer_connections") }
        Label { text: qsTr("Cache statistics"); font.bold: true; Layout.columnSpan: 2; Layout.topMargin: 8 }
        Label { text: qsTr("Read cache hits") }
        Label { text: root.raw("read_cache_hits") + "%" }
        Label { text: qsTr("Total buffers size") }
        Label { text: root.size("total_buffers_size") }
        Label { text: qsTr("Performance statistics"); font.bold: true; Layout.columnSpan: 2; Layout.topMargin: 8 }
        Label { text: qsTr("Write cache overload") }
        Label { text: root.raw("write_cache_overload") + "%" }
        Label { text: qsTr("Read cache overload") }
        Label { text: root.raw("read_cache_overload") + "%" }
        Label { text: qsTr("Queued I/O jobs") }
        Label { text: root.raw("queued_io_jobs") }
        Label { text: qsTr("Average time in queue") }
        Label { text: root.raw("average_time_queue") + " ms" }
        Label { text: qsTr("Total queued size") }
        Label { text: root.size("total_queued_size") }
        Label { text: qsTr("Queued tracker announces") }
        Label { text: root.raw("queued_tracker_announces") }
        Label { text: qsTr("Request latency") }
        Label { text: root.raw("request_latency") + " ms" }
    }
}

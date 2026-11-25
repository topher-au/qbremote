import QtQuick
import QtQuick.Controls
import org.qbremote

// Global download/upload speed history (buffered in Rust, one sample per sync).
Item {
    id: root
    property string history: AppController.speedHistory
    onHistoryChanged: canvas.requestPaint()
    Canvas {
        id: canvas
        anchors.fill: parent
        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0, 0, width, height)
            ctx.fillStyle = "#00000010"
            ctx.fillRect(0, 0, width, height)
            var pts = root.history.length ? root.history.split(";").map(function (s) { return s.split(",").map(Number) }) : []
            if (pts.length < 2) return
            var mx = 1024
            for (var i = 0; i < pts.length; ++i) mx = Math.max(mx, pts[i][0], pts[i][1])
            ctx.strokeStyle = "#88888855"
            for (var g = 1; g < 4; ++g) { ctx.beginPath(); ctx.moveTo(0, height * g / 4); ctx.lineTo(width, height * g / 4); ctx.stroke() }
            function draw(idx, color) {
                ctx.strokeStyle = color
                ctx.lineWidth = 1.5
                ctx.beginPath()
                for (var i = 0; i < pts.length; ++i) {
                    var x = width * i / 299, y = height - (height - 4) * pts[i][idx] / mx - 2
                    if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y)
                }
                ctx.stroke()
            }
            draw(0, "#2f7fd1")
            draw(1, "#e67e22")
        }
    }
    Label { anchors { left: parent.left; top: parent.top; margins: 4 } text: qsTr("↓ %1   ↑ %2").arg(AppController.dlSpeed).arg(AppController.ulSpeed); font.pixelSize: 11 }
    Label { anchors { right: parent.right; top: parent.top; margins: 4 } text: qsTr("Limits ↓ %1  ↑ %2").arg(AppController.dlLimit).arg(AppController.ulLimit); font.pixelSize: 11; opacity: 0.7 }
}

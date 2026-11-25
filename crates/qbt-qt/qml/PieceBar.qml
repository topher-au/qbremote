import QtQuick
import QtQuick.Controls

// Piece-state bar (states: 0 missing, 1 downloading, 2 have) or availability bar.
Item {
    id: root
    property string states: ""            // digits for state mode
    property string values: ""            // comma-separated counts for availability mode
    property bool availability: false
    implicitHeight: 18
    onStatesChanged: canvas.requestPaint()
    onValuesChanged: canvas.requestPaint()
    onWidthChanged: canvas.requestPaint()

    Rectangle { anchors.fill: parent; color: root.palette ? root.palette.base : "white"; border.color: "#888"; border.width: 1 }
    Canvas {
        id: canvas
        anchors { fill: parent; margins: 1 }
        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0, 0, width, height)
            var n, get
            if (root.availability) {
                var a = root.values.length ? root.values.split(",").map(Number) : []
                n = a.length
                var mx = 1
                for (var i = 0; i < n; ++i) mx = Math.max(mx, a[i])
                get = function (i) { return a[i] / mx }
            } else {
                n = root.states.length
                get = function (i) { var c = root.states.charCodeAt(i) - 48; return c === 2 ? 1 : (c === 1 ? 0.5 : 0) }
            }
            if (n === 0 || width < 1) return
            var w = Math.floor(width)
            for (var x = 0; x < w; ++x) {
                var from = Math.floor(x * n / w), to = Math.max(from + 1, Math.floor((x + 1) * n / w))
                var sum = 0
                for (var k = from; k < to && k < n; ++k) sum += get(k)
                var f = sum / (to - from)
                if (f <= 0) continue
                ctx.fillStyle = root.availability ? Qt.rgba(0.2, 0.55, 0.9, 0.3 + 0.7 * f) : (f >= 0.99 ? "#2f7fd1" : Qt.rgba(0.2, 0.7, 0.3, 0.3 + 0.7 * f))
                ctx.fillRect(x, 0, 1, height)
            }
        }
    }
}

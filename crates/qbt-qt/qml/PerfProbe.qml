import QtQuick
import org.qbremote

// Counts rendered frames while live updates stream in and prints the average rate.
Item {
    id: root
    required property int seconds
    property int frames: 0
    property real maxGap: 0
    property real last: 0
    property int elapsed: 0
    FrameAnimation {
        running: AppController.connectionState === "connected"
        onTriggered: {
            root.frames++
            var now = Date.now()
            if (root.last > 0) root.maxGap = Math.max(root.maxGap, now - root.last)
            root.last = now
        }
    }
    Timer {
        interval: 1000
        repeat: true
        running: AppController.connectionState === "connected"
        onTriggered: {
            root.elapsed++
            if (root.elapsed === 3) { root.frames = 0; root.maxGap = 0 }   // skip warm-up
            if (root.elapsed >= root.seconds) {
                var secs = root.elapsed - 3
                console.log("perf: " + (root.frames / secs).toFixed(1) + " frames/s over " + secs + " s, worst frame gap " + root.maxGap + " ms")
                Qt.quit()
            }
        }
    }
}

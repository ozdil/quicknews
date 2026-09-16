import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import "qml/theme"

Scope {
    id: root

    // Bar Widget Button for Omarchy Top Bar
    Rectangle {
        id: barBtn
        implicitWidth: 36
        implicitHeight: 28
        radius: 6
        color: btnMouse.containsMouse ? Theme.bgCardHover : "transparent"

        RowLayout {
            anchors.centerIn: parent
            spacing: 4

            Text {
                text: "\uf1ea"
                font.family: Theme.iconFont
                font.pixelSize: 14
                color: btnMouse.containsMouse ? Theme.accent : Theme.textMain
            }
        }

        MouseArea {
            id: btnMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: {
                toggleProc.running = true;
            }
        }
    }

    Process {
        id: toggleProc
        command: ["quicknews"]
    }
}

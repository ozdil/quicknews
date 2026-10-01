import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property string appVersion: "v0.2.0"
    signal closeRequested()

    color: Qt.rgba(0, 0, 0, 0.70)

    MouseArea {
        anchors.fill: parent
        onClicked: root.closeRequested()
    }

    Rectangle {
        width: Math.min(parent.width - 48, 620)
        implicitHeight: modalCol.implicitHeight + 44
        anchors.centerIn: parent
        radius: Theme.radiusMd
        color: Theme.bgCard
        border.color: Theme.borderLight
        border.width: 1

        MouseArea {
            anchors.fill: parent
            // Prevent close on inside click
        }

        ColumnLayout {
            id: modalCol
            anchors.fill: parent
            anchors.margins: 22
            spacing: 16

            // Modal Header
            RowLayout {
                Layout.fillWidth: true
                spacing: 10

                Rectangle {
                    width: 36
                    height: 36
                    radius: Theme.radiusSm
                    color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.15)
                    border.color: Theme.accent
                    border.width: 1

                    Text {
                        anchors.centerIn: parent
                        text: Theme.iconNews
                        font.family: Theme.iconFont
                        font.pixelSize: 18
                        color: Theme.accent
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    RowLayout {
                        spacing: 8

                        Text {
                            text: I18n.t("info_title")
                            font.family: Theme.fontFamily
                            font.pixelSize: 16
                            font.bold: true
                            color: Theme.textMain
                        }

                        Rectangle {
                            height: 18
                            width: verText.implicitWidth + 10
                            radius: 4
                            color: Qt.rgba(Theme.accentGreen.r, Theme.accentGreen.g, Theme.accentGreen.b, 0.15)
                            border.color: Theme.accentGreen
                            border.width: 1

                            Text {
                                id: verText
                                anchors.centerIn: parent
                                text: root.appVersion
                                font.family: Theme.fontFamily
                                font.pixelSize: 10
                                font.bold: true
                                color: Theme.accentGreen
                            }
                        }
                    }

                    Text {
                        text: I18n.t("info_subtitle")
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        color: Theme.textMuted
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }

                // Close Button
                Rectangle {
                    width: 28
                    height: 28
                    radius: Theme.radiusSm
                    color: closeMouse.containsMouse ? Theme.bgCardHover : "transparent"

                    Text {
                        anchors.centerIn: parent
                        text: Theme.iconTimes
                        font.family: Theme.iconFont
                        font.pixelSize: 13
                        color: closeMouse.containsMouse ? Theme.accentRed : Theme.textMuted
                    }

                    MouseArea {
                        id: closeMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.closeRequested()
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: Theme.border
            }

            // Architecture Section
            RowLayout {
                Layout.fillWidth: true
                spacing: 12

                Text {
                    text: Theme.iconSliders
                    font.family: Theme.iconFont
                    font.pixelSize: 14
                    color: Theme.accentCyan
                    Layout.alignment: Qt.AlignTop
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 3

                    Text {
                        text: I18n.t("info_architecture")
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        font.bold: true
                        color: Theme.textMain
                    }

                    Text {
                        text: I18n.t("info_arch_desc")
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        color: Theme.textMuted
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
            }

            // Security Section
            RowLayout {
                Layout.fillWidth: true
                spacing: 12

                Text {
                    text: Theme.iconShield
                    font.family: Theme.iconFont
                    font.pixelSize: 14
                    color: Theme.accentGreen
                    Layout.alignment: Qt.AlignTop
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 3

                    Text {
                        text: I18n.t("info_security")
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        font.bold: true
                        color: Theme.textMain
                    }

                    Text {
                        text: I18n.t("info_security_desc")
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        color: Theme.textMuted
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }
            }

            // Shortcuts Section
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6

                Text {
                    text: I18n.t("info_shortcuts")
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    font.bold: true
                    color: Theme.textMain
                }

                GridLayout {
                    columns: 2
                    rowSpacing: 4
                    columnSpacing: 16
                    Layout.fillWidth: true

                    // Row 1
                    Text { text: "J / Aşağı : Sonraki haber"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }
                    Text { text: "K / Yukarı : Önceki haber"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }

                    // Row 2
                    Text { text: "F : Zen / Tam Ekran okuma"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }
                    Text { text: "S : Haberi kaydet / çıkar"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }

                    // Row 3
                    Text { text: "R : Akışları senkronize et"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }
                    Text { text: "/ : Başlıklarda arama"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }

                    // Row 4
                    Text { text: "? : Bilgi / Hakkında paneli"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.accent }
                    Text { text: "Esc : Pencere / Zen modundan çık"; font.family: Theme.fontFamily; font.pixelSize: 11; color: Theme.textMuted }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: Theme.border
            }

            // Support & Buy Me a Coffee Section
            Rectangle {
                Layout.fillWidth: true
                height: 44
                radius: Theme.radiusSm
                color: Theme.bgSurface
                border.color: Theme.border
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12
                    spacing: 12

                    Text {
                        text: Theme.iconCoffee
                        font.family: Theme.iconFont
                        font.pixelSize: 14
                        color: "#FFDD00"
                    }

                    Text {
                        text: I18n.t("info_support")
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        color: Theme.textMain
                        Layout.fillWidth: true
                    }

                    // Buy Me a Coffee Button
                    Rectangle {
                        height: 28
                        width: bmacRow.implicitWidth + 16
                        radius: Theme.radiusSm
                        color: bmacMouse.containsMouse ? "#FFE433" : "#FFDD00"

                        RowLayout {
                            id: bmacRow
                            anchors.centerIn: parent
                            spacing: 6

                            Text {
                                text: Theme.iconCoffee
                                font.family: Theme.iconFont
                                font.pixelSize: 11
                                color: "#000000"
                            }

                            Text {
                                text: "Buy Me a Coffee"
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                font.bold: true
                                color: "#000000"
                            }
                        }

                        MouseArea {
                            id: bmacMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: Qt.openUrlExternally("https://buymeacoffee.com/ozdil")
                        }
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 1
                color: Theme.border
            }

            // Footer Button
            RowLayout {
                Layout.fillWidth: true

                Text {
                    text: "GitHub: ozdil/quicknews"
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    color: ghMouse.containsMouse ? Theme.accent : Theme.textDim

                    MouseArea {
                        id: ghMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Qt.openUrlExternally("https://github.com/ozdil/quicknews")
                    }
                }

                Item { Layout.fillWidth: true }

                Rectangle {
                    height: 30
                    width: 100
                    radius: Theme.radiusSm
                    color: okBtnMouse.containsMouse ? Theme.accentHover : Theme.accent

                    Text {
                        anchors.centerIn: parent
                        text: I18n.t("info_close")
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        font.bold: true
                        color: Theme.bgDark
                    }

                    MouseArea {
                        id: okBtnMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.closeRequested()
                    }
                }
            }
        }
    }
}

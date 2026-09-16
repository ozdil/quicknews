import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property bool isSearching: false
    property string statusMessage: ""

    signal closeRequested()
    signal promptSubmitted(string prompt)

    color: Qt.rgba(0, 0, 0, 0.65)

    MouseArea {
        anchors.fill: parent
        onClicked: {
            if (!root.isSearching) root.closeRequested();
        }
    }

    Rectangle {
        width: Math.min(parent.width - 40, 640)
        implicitHeight: modalCol.implicitHeight + 40
        anchors.centerIn: parent
        radius: Theme.radiusMd
        color: Theme.bgCard
        border.color: Theme.borderLight
        border.width: 1

        MouseArea {
            anchors.fill: parent
            // Prevent clicks inside modal from closing it
        }

        ColumnLayout {
            id: modalCol
            anchors.fill: parent
            anchors.margins: 20
            spacing: 14

            // Modal Header
            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Text {
                    text: Theme.iconAi
                    font.family: Theme.iconFont
                    font.pixelSize: 18
                    color: Theme.accent
                }

                Text {
                    text: "Yapay Zeka ile Kaynak Ekle"
                    font.family: Theme.fontFamily
                    font.pixelSize: 15
                    font.bold: true
                    color: Theme.textMain
                    Layout.fillWidth: true
                }

                Text {
                    text: Theme.iconTimes
                    font.family: Theme.iconFont
                    font.pixelSize: 14
                    color: Theme.textMuted
                    enabled: !root.isSearching

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.closeRequested()
                    }
                }
            }

            Text {
                text: "URL veya RSS adresiyle ugrasmayin. Eklemek istediginiz haber sitelerini dogal dille yazin, yapay zeka siteleri bulup akislarini otomatik baglasin."
                font.family: Theme.fontFamily
                font.pixelSize: 11
                color: Theme.textMuted
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            // Text Input Box
            Rectangle {
                Layout.fillWidth: true
                height: 42
                radius: Theme.radiusSm
                color: Theme.bgSurface
                border.color: promptInput.activeFocus ? Theme.accent : Theme.border
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 12
                    anchors.rightMargin: 12

                    TextInput {
                        id: promptInput
                        Layout.fillWidth: true
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        color: Theme.textMain
                        selectByMouse: true
                        clip: true
                        enabled: !root.isSearching
                        onAccepted: {
                            if (text.trim().length > 0) {
                                root.promptSubmitted(text.trim());
                            }
                        }

                        Text {
                            anchors.fill: parent
                            text: "Orn: Turkiye'deki en iyi teknoloji sayfalarindan 10 tanesini ekle"
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            color: Theme.textDim
                            visible: !promptInput.text && !promptInput.activeFocus
                        }
                    }
                }
            }

            // Quick Example Prompts
            Text {
                text: "HAZIR ORNEKLER (Tiklayarak secin):"
                font.family: Theme.fontFamily
                font.pixelSize: 9
                font.bold: true
                color: Theme.textDim
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 6

                Repeater {
                    model: [
                        "Turkiye'deki en iyi teknoloji sayfalarindan 10 tanesini ekle",
                        "Acik kaynak ve Linux odakli 5 blog ekle",
                        "Populer bilim ve uzay haber sitelerini ekle",
                        "Ekonomi ve finans haber kaynaklarini ekle"
                    ]

                    Rectangle {
                        Layout.fillWidth: true
                        height: 28
                        radius: Theme.radiusSm
                        color: exMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 8

                            Text {
                                text: "+"
                                font.family: Theme.fontFamily
                                font.pixelSize: 12
                                color: Theme.accent
                            }

                            Text {
                                text: modelData
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                color: Theme.textMain
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                        }

                        MouseArea {
                            id: exMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            enabled: !root.isSearching
                            onClicked: {
                                promptInput.text = modelData;
                            }
                        }
                    }
                }
            }

            // Status feedback message
            Text {
                visible: root.statusMessage.length > 0
                text: root.statusMessage
                font.family: Theme.fontFamily
                font.pixelSize: 11
                color: root.isSearching ? Theme.accentCyan : Theme.accentGreen
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            // Submit Button
            Rectangle {
                Layout.fillWidth: true
                height: 38
                radius: Theme.radiusSm
                color: root.isSearching ? Theme.bgSurface : (submitMouse.containsMouse ? Theme.accentHover : Theme.accent)

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 8

                    Text {
                        text: root.isSearching ? Theme.iconRefresh : Theme.iconAi
                        font.family: Theme.iconFont
                        font.pixelSize: 13
                        color: root.isSearching ? Theme.textMuted : Theme.bgDark
                    }

                    Text {
                        text: root.isSearching ? "Siteler Araniyor ve Dogrulaniyor..." : "Kaynaklari Bul ve Ekle"
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        font.bold: true
                        color: root.isSearching ? Theme.textMuted : Theme.bgDark
                    }
                }

                MouseArea {
                    id: submitMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    enabled: !root.isSearching
                    onClicked: {
                        if (promptInput.text.trim().length > 0) {
                            root.promptSubmitted(promptInput.text.trim());
                        }
                    }
                }
            }
        }
    }
}

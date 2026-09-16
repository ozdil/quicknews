import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property var sources: []
    property string activeCategory: "Tümü"
    property string selectedSourceId: ""
    property int unreadCount: 0

    signal categorySelected(string category)
    signal sourceSelected(string sourceId)
    signal openAddModal()
    signal refreshRequested()

    color: Theme.bgDark
    border.color: Theme.border
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 12

        // Top App Header
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Text {
                text: Theme.iconNews
                font.family: Theme.iconFont
                font.pixelSize: 18
                color: Theme.accent
            }

            Text {
                text: "QuickNews"
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.bold: true
                color: Theme.textMain
                Layout.fillWidth: true
            }

            // Sync/Refresh Button
            Rectangle {
                width: 28
                height: 28
                radius: Theme.radiusSm
                color: refreshMouse.containsMouse ? Theme.bgCardHover : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: Theme.iconRefresh
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: refreshMouse.containsMouse ? Theme.accent : Theme.textMuted
                }

                MouseArea {
                    id: refreshMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.refreshRequested()
                }
            }
        }

        // Add Source Button (Natural Language AI Prompt)
        Rectangle {
            Layout.fillWidth: true
            height: 34
            radius: Theme.radiusSm
            color: addBtnMouse.containsMouse ? Theme.accentHover : Theme.accent

            RowLayout {
                anchors.centerIn: parent
                spacing: 8

                Text {
                    text: Theme.iconAi
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: Theme.bgDark
                }

                Text {
                    text: "AI ile Kaynak Ekle"
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    font.bold: true
                    color: Theme.bgDark
                }
            }

            MouseArea {
                id: addBtnMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.openAddModal()
            }
        }

        // Categories Title
        Text {
            text: "KATEGORILER"
            font.family: Theme.fontFamily
            font.pixelSize: 10
            font.bold: true
            color: Theme.textDim
            Layout.topMargin: 4
        }

        // Categories Pills
        Flow {
            Layout.fillWidth: true
            spacing: 6

            Repeater {
                model: ["Tümü", "Teknoloji", "Linux", "Bilim", "Ekonomi"]

                Rectangle {
                    width: catText.implicitWidth + 16
                    height: 24
                    radius: Theme.radiusSm
                    color: root.activeCategory === modelData ? Theme.accent : (catMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface)

                    Text {
                        id: catText
                        anchors.centerIn: parent
                        text: modelData
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        color: root.activeCategory === modelData ? Theme.bgDark : Theme.textMain
                    }

                    MouseArea {
                        id: catMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            root.activeCategory = modelData;
                            root.categorySelected(modelData);
                        }
                    }
                }
            }
        }

        // Sources List Title & Counter
        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: 8

            Text {
                text: "KAYNAKLAR (" + root.sources.length + ")"
                font.family: Theme.fontFamily
                font.pixelSize: 10
                font.bold: true
                color: Theme.textDim
                Layout.fillWidth: true
            }

            Text {
                visible: root.unreadCount > 0
                text: root.unreadCount + " yeni"
                font.family: Theme.fontFamily
                font.pixelSize: 10
                font.bold: true
                color: Theme.accentOrange
            }
        }

        // Scrollable Sources List
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: sourceListView
                width: parent.width
                model: root.sources
                spacing: 4

                delegate: Rectangle {
                    required property var modelData
                    width: sourceListView.width
                    height: 32
                    radius: Theme.radiusSm
                    color: root.selectedSourceId === modelData.id ? Theme.bgCard : (srcMouse.containsMouse ? Theme.bgCardHover : "transparent")

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 8
                        anchors.rightMargin: 8
                        spacing: 8

                        Text {
                            text: Theme.iconFeed
                            font.family: Theme.iconFont
                            font.pixelSize: 11
                            color: root.selectedSourceId === modelData.id ? Theme.accent : Theme.textMuted
                        }

                        Text {
                            text: modelData.name
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            color: root.selectedSourceId === modelData.id ? Theme.textMain : Theme.textMuted
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }

                        Text {
                            text: modelData.category
                            font.family: Theme.fontFamily
                            font.pixelSize: 9
                            color: Theme.textDim
                        }
                    }

                    MouseArea {
                        id: srcMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (root.selectedSourceId === modelData.id) {
                                root.selectedSourceId = "";
                                root.sourceSelected("");
                            } else {
                                root.selectedSourceId = modelData.id;
                                root.sourceSelected(modelData.id);
                            }
                        }
                    }
                }
            }
        }

        // Bottom Footer info
        Text {
            Layout.fillWidth: true
            text: "Omarchy Linux • Resimsiz ve Guvenli"
            font.family: Theme.fontFamily
            font.pixelSize: 9
            color: Theme.textDim
            horizontalAlignment: Text.AlignHCenter
        }
    }
}

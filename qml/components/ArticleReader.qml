import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property var currentArticle: null
    property var fullCleanArticle: null
    property var aiSummary: null
    property bool isLoadingContent: false
    property bool isLoadingAi: false
    property int readerFontSize: 14

    signal summarizeRequested(string articleUrl)
    signal openExternalRequested(string articleUrl)

    color: Theme.bgBase
    border.color: Theme.border
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        // Top Toolbar
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            Text {
                text: Theme.iconBook
                font.family: Theme.iconFont
                font.pixelSize: 14
                color: Theme.accent
            }

            Text {
                text: "SAF METIN OKUYUCU"
                font.family: Theme.fontFamily
                font.pixelSize: 11
                font.bold: true
                color: Theme.textDim
            }

            Item { Layout.fillWidth: true }

            // AI Summarize Button
            Rectangle {
                visible: root.currentArticle !== null
                height: 28
                width: aiRow.implicitWidth + 16
                radius: Theme.radiusSm
                color: aiMouse.containsMouse ? Theme.accentHover : Theme.accentPurple

                RowLayout {
                    id: aiRow
                    anchors.centerIn: parent
                    spacing: 6

                    Text {
                        text: root.isLoadingAi ? Theme.iconRefresh : Theme.iconAi
                        font.family: Theme.iconFont
                        font.pixelSize: 12
                        color: Theme.bgDark
                    }

                    Text {
                        text: root.isLoadingAi ? "Ozetleniyor..." : "Yapay Zeka Ozeti"
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        font.bold: true
                        color: Theme.bgDark
                    }
                }

                MouseArea {
                    id: aiMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    enabled: !root.isLoadingAi
                    onClicked: {
                        if (root.currentArticle && root.currentArticle.link) {
                            root.summarizeRequested(root.currentArticle.link);
                        }
                    }
                }
            }

            // Font Size Decrement
            Rectangle {
                width: 28
                height: 28
                radius: Theme.radiusSm
                color: fontMinusMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface

                Text {
                    anchors.centerIn: parent
                    text: "A-"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    font.bold: true
                    color: Theme.textMain
                }

                MouseArea {
                    id: fontMinusMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (root.readerFontSize > 11) root.readerFontSize -= 1;
                    }
                }
            }

            // Font Size Increment
            Rectangle {
                width: 28
                height: 28
                radius: Theme.radiusSm
                color: fontPlusMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface

                Text {
                    anchors.centerIn: parent
                    text: "A+"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    font.bold: true
                    color: Theme.textMain
                }

                MouseArea {
                    id: fontPlusMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (root.readerFontSize < 24) root.readerFontSize += 1;
                    }
                }
            }

            // Open in external browser
            Rectangle {
                visible: root.currentArticle !== null
                width: 28
                height: 28
                radius: Theme.radiusSm
                color: extMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface

                Text {
                    anchors.centerIn: parent
                    text: Theme.iconExternalLink
                    font.family: Theme.iconFont
                    font.pixelSize: 12
                    color: extMouse.containsMouse ? Theme.accent : Theme.textMuted
                }

                MouseArea {
                    id: extMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (root.currentArticle && root.currentArticle.link) {
                            root.openExternalRequested(root.currentArticle.link);
                        }
                    }
                }
            }
        }

        // Horizontal line
        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.border
        }

        // Empty state when no article selected
        Item {
            visible: root.currentArticle === null
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 12

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: Theme.iconNews
                    font.family: Theme.iconFont
                    font.pixelSize: 48
                    color: Theme.textDim
                }

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: "Okumak istediginiz haberi listeden secin"
                    font.family: Theme.fontFamily
                    font.pixelSize: 14
                    color: Theme.textMuted
                }

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: "Sifir reklam • Sifir resim • Tamamen saf metin"
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    color: Theme.textDim
                }
            }
        }

        // Article Content ScrollArea
        ScrollView {
            visible: root.currentArticle !== null
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            Flickable {
                id: flickable
                contentWidth: readerCol.width
                contentHeight: readerCol.implicitHeight + 60
                boundsBehavior: Flickable.StopAtBounds

                ColumnLayout {
                    id: readerCol
                    width: Math.min(flickable.width - 24, 820)
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 16

                    // Article Title (Clean, Bold, Large)
                    Text {
                        text: (root.fullCleanArticle && root.fullCleanArticle.title) ? root.fullCleanArticle.title : (root.currentArticle ? root.currentArticle.title : "")
                        font.family: Theme.fontFamily
                        font.pixelSize: root.readerFontSize + 8
                        font.bold: true
                        color: Theme.textMain
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }

                    // Metadata Row
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 10

                        Text {
                            text: (root.currentArticle ? root.currentArticle.source_name : "")
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            font.bold: true
                            color: Theme.accent
                        }

                        Text {
                            visible: root.fullCleanArticle && root.fullCleanArticle.author
                            text: "• " + (root.fullCleanArticle ? root.fullCleanArticle.author : "")
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            color: Theme.textMuted
                        }

                        Text {
                            visible: root.fullCleanArticle && root.fullCleanArticle.published_date
                            text: "• " + (root.fullCleanArticle ? root.fullCleanArticle.published_date : "")
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            color: Theme.textDim
                        }

                        Text {
                            text: "• " + (root.fullCleanArticle ? root.fullCleanArticle.reading_time_mins : (root.currentArticle ? root.currentArticle.reading_time_mins : 1)) + " dk okuma"
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            color: Theme.accentGreen
                        }
                    }

                    // AI Summary Card (if generated)
                    Rectangle {
                        visible: root.aiSummary !== null
                        Layout.fillWidth: true
                        implicitHeight: aiSummaryCol.implicitHeight + 24
                        radius: Theme.radiusMd
                        color: Theme.bgCard
                        border.color: Theme.accentPurple
                        border.width: 1

                        ColumnLayout {
                            id: aiSummaryCol
                            anchors.fill: parent
                            anchors.margins: 14
                            spacing: 10

                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8

                                Text {
                                    text: Theme.iconAi
                                    font.family: Theme.iconFont
                                    font.pixelSize: 14
                                    color: Theme.accentPurple
                                }

                                Text {
                                    text: "YAPAY ZEKA HABER ANALIZI"
                                    font.family: Theme.fontFamily
                                    font.pixelSize: 11
                                    font.bold: true
                                    color: Theme.accentPurple
                                    Layout.fillWidth: true
                                }
                            }

                            // Neutral Title
                            Text {
                                visible: root.aiSummary && root.aiSummary.neutral_title
                                text: "Notr Baslik: " + (root.aiSummary ? root.aiSummary.neutral_title : "")
                                font.family: Theme.fontFamily
                                font.pixelSize: 12
                                font.bold: true
                                color: Theme.textMain
                                wrapMode: Text.WordWrap
                                Layout.fillWidth: true
                            }

                            // 3 Bullet Points
                            Repeater {
                                model: root.aiSummary ? root.aiSummary.key_points : []

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    Text {
                                        text: "•"
                                        font.family: Theme.fontFamily
                                        font.pixelSize: 14
                                        font.bold: true
                                        color: Theme.accentCyan
                                        Layout.alignment: Qt.AlignTop
                                    }

                                    Text {
                                        text: modelData
                                        font.family: Theme.fontFamily
                                        font.pixelSize: 12
                                        color: Theme.textMain
                                        wrapMode: Text.WordWrap
                                        Layout.fillWidth: true
                                    }
                                }
                            }
                        }
                    }

                    // Loading indicator for article body
                    RowLayout {
                        visible: root.isLoadingContent
                        Layout.fillWidth: true
                        spacing: 8

                        Text {
                            text: Theme.iconRefresh
                            font.family: Theme.iconFont
                            font.pixelSize: 13
                            color: Theme.accent
                        }

                        Text {
                            text: "Haberin reklamsiz ve saf metni cikariliyor..."
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            color: Theme.accent
                        }
                    }

                    // Clean Article Body Text (Pure typography, high readability, markdown formatted)
                    TextEdit {
                        id: contentTextEdit
                        Layout.fillWidth: true
                        text: (root.fullCleanArticle && root.fullCleanArticle.content_text) ? root.fullCleanArticle.content_text : (root.currentArticle ? root.currentArticle.summary : "")
                        font.family: Theme.fontFamily
                        font.pixelSize: root.readerFontSize
                        color: Theme.textMain
                        wrapMode: Text.WordWrap
                        readOnly: true
                        selectByMouse: true
                        textFormat: TextEdit.MarkdownText
                    }

                    Item { height: 40 }
                }
            }
        }
    }
}

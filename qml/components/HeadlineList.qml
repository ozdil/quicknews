import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property var articles: []
    property string activeCategory: "Tümü"
    property string activeSourceId: ""
    property string selectedArticleId: ""
    property string searchQuery: ""

    signal articleSelected(var article)

    color: Theme.bgBase
    border.color: Theme.border
    border.width: 1

    function getFilteredArticles() {
        if (!root.articles || root.articles.length === 0) return [];
        var res = [];
        var q = root.searchQuery.trim().toLowerCase();

        for (var i = 0; i < root.articles.length; i++) {
            var a = root.articles[i];
            if (root.activeCategory !== "Tümü" && a.category !== root.activeCategory) {
                continue;
            }
            if (root.activeSourceId && a.source_id !== root.activeSourceId) {
                continue;
            }
            if (q.length > 0) {
                var matchTitle = a.title && a.title.toLowerCase().indexOf(q) !== -1;
                var matchSummary = a.summary && a.summary.toLowerCase().indexOf(q) !== -1;
                var matchSource = a.source_name && a.source_name.toLowerCase().indexOf(q) !== -1;
                if (!matchTitle && !matchSummary && !matchSource) continue;
            }
            res.push(a);
        }
        return res;
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        // Search bar
        Rectangle {
            Layout.fillWidth: true
            height: 36
            radius: Theme.radiusSm
            color: Theme.bgSurface
            border.color: searchInput.activeFocus ? Theme.accent : Theme.border
            border.width: 1

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                spacing: 8

                Text {
                    text: Theme.iconSearch
                    font.family: Theme.iconFont
                    font.pixelSize: 12
                    color: Theme.textDim
                }

                TextInput {
                    id: searchInput
                    Layout.fillWidth: true
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    color: Theme.textMain
                    selectByMouse: true
                    clip: true
                    onTextChanged: root.searchQuery = text

                    Text {
                        anchors.fill: parent
                        text: "Haberlerde ara..."
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        color: Theme.textDim
                        visible: !searchInput.text && !searchInput.activeFocus
                    }
                }

                Text {
                    visible: searchInput.text.length > 0
                    text: Theme.iconTimes
                    font.family: Theme.iconFont
                    font.pixelSize: 11
                    color: Theme.textMuted

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            searchInput.text = "";
                            root.searchQuery = "";
                        }
                    }
                }
            }
        }

        // Article count header
        RowLayout {
            Layout.fillWidth: true

            Text {
                text: "HABER AKISI (" + articleListView.count + ")"
                font.family: Theme.fontFamily
                font.pixelSize: 10
                font.bold: true
                color: Theme.textDim
                Layout.fillWidth: true
            }

            Text {
                text: root.activeCategory
                font.family: Theme.fontFamily
                font.pixelSize: 10
                color: Theme.accentCyan
            }
        }

        // Articles ListView
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: articleListView
                width: parent.width
                model: root.getFilteredArticles()
                spacing: 8

                delegate: Rectangle {
                    required property var modelData
                    width: articleListView.width
                    implicitHeight: itemCol.implicitHeight + 20
                    radius: Theme.radiusSm
                    color: root.selectedArticleId === modelData.id ? Theme.bgCard : (itemMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface)
                    border.color: root.selectedArticleId === modelData.id ? Theme.accent : Theme.border
                    border.width: 1

                    ColumnLayout {
                        id: itemCol
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 6

                        // Source and Metadata Row
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 6

                            // Unread indicator dot
                            Rectangle {
                                width: 6
                                height: 6
                                radius: 3
                                color: Theme.accentOrange
                                visible: !modelData.is_read
                            }

                            Text {
                                text: modelData.source_name
                                font.family: Theme.fontFamily
                                font.pixelSize: 11
                                font.bold: true
                                color: Theme.accent
                            }

                            Text {
                                text: "•"
                                font.family: Theme.fontFamily
                                font.pixelSize: 10
                                color: Theme.textDim
                            }

                            Text {
                                text: modelData.reading_time_mins + " dk okuma"
                                font.family: Theme.fontFamily
                                font.pixelSize: 10
                                color: Theme.textDim
                            }

                            Item { Layout.fillWidth: true }

                            Text {
                                text: modelData.category
                                font.family: Theme.fontFamily
                                font.pixelSize: 9
                                color: Theme.textMuted
                            }
                        }

                        // Headline Title (Pure Typography)
                        Text {
                            text: modelData.title
                            font.family: Theme.fontFamily
                            font.pixelSize: 13
                            font.bold: !modelData.is_read
                            color: modelData.is_read ? Theme.textMuted : Theme.textMain
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                            maximumLineCount: 3
                            elide: Text.ElideRight
                        }

                        // Short clean snippet
                        Text {
                            visible: modelData.summary && modelData.summary.length > 0
                            text: modelData.summary
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            color: Theme.textDim
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                            maximumLineCount: 2
                            elide: Text.ElideRight
                        }
                    }

                    MouseArea {
                        id: itemMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            root.selectedArticleId = modelData.id;
                            root.articleSelected(modelData);
                        }
                    }
                }
            }
        }
    }
}

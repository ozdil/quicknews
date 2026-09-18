import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property var articles: []
    property var savedArticles: []
    property string activeCategory: "Tümü"
    property string activeSourceId: ""
    property string selectedArticleId: ""
    property string searchQuery: ""
    property string statusFilter: "Tümü"
    property string timeFilter: "Tümü"
    readonly property bool isSearchFocused: searchInput.activeFocus

    signal articleSelected(var article)
    signal tagSelected(string tag)

    color: Theme.bgBase
    border.color: Theme.border
    border.width: 1

    function clearSearch() {
        searchInput.text = "";
        root.searchQuery = "";
        root.focus = true;
    }

    function isSaved(id, link) {
        if (!root.savedArticles) return false;
        for (var i = 0; i < root.savedArticles.length; i++) {
            if (root.savedArticles[i].id === id || root.savedArticles[i].link === link) return true;
        }
        return false;
    }

    function getFilteredArticles() {
        if (!root.articles || root.articles.length === 0) return [];
        var res = [];
        var q = root.searchQuery.trim().toLowerCase();

        for (var i = 0; i < root.articles.length; i++) {
            var a = root.articles[i];
            if (root.activeCategory !== "Tümü") {
                var c = root.activeCategory.toLowerCase();
                var aCat = (a.category || "").toLowerCase();
                var matchCat = false;
                if (c === "linux") {
                    matchCat = aCat.indexOf("linux") !== -1 || (a.source_name && a.source_name.toLowerCase().indexOf("phoronix") !== -1);
                } else if (c === "teknoloji") {
                    matchCat = aCat.indexOf("teknoloji") !== -1 || aCat.indexOf("bilişim") !== -1 || aCat.indexOf("dijital") !== -1;
                } else if (c === "donanım" || c === "donanim") {
                    matchCat = aCat.indexOf("donan") !== -1 || (a.source_name && a.source_name.toLowerCase().indexOf("hwp") !== -1);
                } else if (c === "bilim") {
                    matchCat = aCat.indexOf("bilim") !== -1 || (a.source_name && a.source_name.toLowerCase().indexOf("evrim") !== -1);
                } else if (c === "girişimcilik" || c === "girisimcilik" || c === "girişim") {
                    matchCat = aCat.indexOf("girişim") !== -1 || aCat.indexOf("girisim") !== -1;
                } else {
                    matchCat = (aCat === c || aCat.indexOf(c) !== -1);
                }
                if (!matchCat && a.tags && a.tags.length > 0) {
                    for (var t = 0; t < a.tags.length; t++) {
                        var tagVal = (a.tags[t] || "").toLowerCase();
                        if (tagVal.indexOf(c) !== -1 || c.indexOf(tagVal) !== -1) {
                            matchCat = true;
                            break;
                        }
                    }
                }
                if (!matchCat) continue;
            }
            if (root.activeSourceId && a.source_id !== root.activeSourceId) {
                continue;
            }
            if (root.statusFilter === "Okunmamış" && a.is_read) {
                continue;
            }
            if (root.statusFilter === "Kaydedilenler" && !root.isSaved(a.id, a.link)) {
                continue;
            }
            if (root.timeFilter === "< 3 dk" && a.reading_time_mins >= 3) {
                continue;
            }
            if (root.timeFilter === "3-6 dk" && (a.reading_time_mins < 3 || a.reading_time_mins > 6)) {
                continue;
            }
            if (root.timeFilter === "> 6 dk" && a.reading_time_mins <= 6) {
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

    function focusSearch() {
        searchInput.forceActiveFocus();
        searchInput.selectAll();
    }

    function selectNext() {
        var list = getFilteredArticles();
        if (list.length === 0) return;
        var curIdx = -1;
        for (var i = 0; i < list.length; i++) {
            if (list[i].id === root.selectedArticleId) {
                curIdx = i;
                break;
            }
        }
        var nextIdx = curIdx + 1;
        if (nextIdx < list.length) {
            root.selectedArticleId = list[nextIdx].id;
            root.articleSelected(list[nextIdx]);
            articleListView.positionViewAtIndex(nextIdx, ListView.Contain);
        }
    }

    function selectPrev() {
        var list = getFilteredArticles();
        if (list.length === 0) return;
        var curIdx = -1;
        for (var i = 0; i < list.length; i++) {
            if (list[i].id === root.selectedArticleId) {
                curIdx = i;
                break;
            }
        }
        var prevIdx = curIdx - 1;
        if (prevIdx >= 0) {
            root.selectedArticleId = list[prevIdx].id;
            root.articleSelected(list[prevIdx]);
            articleListView.positionViewAtIndex(prevIdx, ListView.Contain);
        }
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

        // Status Filter Tabs (Tumu, Okunmamis, Kaydedilenler)
        RowLayout {
            Layout.fillWidth: true
            spacing: 6

            Repeater {
                model: ["Tümü", "Okunmamış", "Kaydedilenler"]

                Rectangle {
                    height: 24
                    Layout.fillWidth: true
                    radius: Theme.radiusSm
                    color: root.statusFilter === modelData ? Theme.accent : (tabMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface)

                    Text {
                        anchors.centerIn: parent
                        text: modelData
                        font.family: Theme.fontFamily
                        font.pixelSize: 10
                        font.bold: root.statusFilter === modelData
                        color: root.statusFilter === modelData ? Theme.bgDark : Theme.textMain
                    }

                    MouseArea {
                        id: tabMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.statusFilter = modelData
                    }
                }
            }
        }

        // Reading Time Filter Pills (< 3 dk, 3-6 dk, > 6 dk)
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Text {
                text: Theme.iconClock
                font.family: Theme.iconFont
                font.pixelSize: 10
                color: Theme.textDim
            }

            Repeater {
                model: ["Tümü", "< 3 dk", "3-6 dk", "> 6 dk"]

                Rectangle {
                    width: timeTxt.implicitWidth + 12
                    height: 20
                    radius: Theme.radiusSm
                    color: root.timeFilter === modelData ? Theme.bgCard : (tMouse.containsMouse ? Theme.bgCardHover : "transparent")
                    border.color: root.timeFilter === modelData ? Theme.accentCyan : Theme.border
                    border.width: 1

                    Text {
                        id: timeTxt
                        anchors.centerIn: parent
                        text: modelData
                        font.family: Theme.fontFamily
                        font.pixelSize: 9
                        color: root.timeFilter === modelData ? Theme.accentCyan : Theme.textMuted
                    }

                    MouseArea {
                        id: tMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.timeFilter = modelData
                    }
                }
            }

            Item { Layout.fillWidth: true }
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
                            textFormat: Text.PlainText
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
                            textFormat: Text.PlainText
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            color: Theme.textDim
                            wrapMode: Text.WordWrap
                            Layout.fillWidth: true
                            maximumLineCount: 2
                            elide: Text.ElideRight
                        }

                        // Automatic Topic & Source Tags Badges
                        Flow {
                            visible: modelData.tags && modelData.tags.length > 0
                            Layout.fillWidth: true
                            spacing: 4

                            Repeater {
                                model: modelData.tags ? modelData.tags : []

                                Rectangle {
                                    id: tagBadge
                                    height: 18
                                    width: tagBadgeText.implicitWidth + 10
                                    radius: 3
                                    color: tagMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface
                                    border.color: tagMouse.containsMouse ? Theme.accentCyan : Theme.border
                                    border.width: 1

                                    Text {
                                        id: tagBadgeText
                                        anchors.centerIn: parent
                                        text: modelData
                                        font.family: Theme.fontFamily
                                        font.pixelSize: 9
                                        font.bold: true
                                        color: tagMouse.containsMouse ? Theme.accent : Theme.accentCyan
                                    }

                                    MouseArea {
                                        id: tagMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: function(mouse) {
                                            mouse.accepted = true;
                                            root.tagSelected(modelData);
                                        }
                                    }
                                }
                            }
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

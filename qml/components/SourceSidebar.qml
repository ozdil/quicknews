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
    property bool isSyncing: false

    signal categorySelected(string category)
    signal sourceSelected(string sourceId)
    signal openAddModal()
    signal openInfoModal()
    signal refreshRequested()
    signal removeSourceRequested(string sourceId, string sourceName)

    color: Theme.bgDark
    border.color: Theme.border
    border.width: 1

    function norm(s) {
        if (!s) return "";
        return String(s).toLowerCase()
            .replace(/ı/g, "i")
            .replace(/ğ/g, "g")
            .replace(/ü/g, "u")
            .replace(/ş/g, "s")
            .replace(/ö/g, "o")
            .replace(/ç/g, "c");
    }

    function isSourceMatchingCategory(source, cat) {
        if (!cat || cat === "Tümü") return true;
        var c = norm(cat);
        var sCat = norm(source.category || "");
        var sName = norm(source.name || "");
        var sDomain = norm(source.domain || "");

        if (c === "gundem") {
            return sCat.indexOf("gundem") !== -1 || sCat.indexOf("genel") !== -1 || sCat.indexOf("ajans") !== -1;
        }
        if (c === "siyaset") {
            return sCat.indexOf("siyaset") !== -1 || sCat.indexOf("politika") !== -1 || sCat.indexOf("meclis") !== -1;
        }
        if (c === "yerel") {
            return sCat.indexOf("yerel") !== -1 || sCat.indexOf("sehir") !== -1 || sCat.indexOf("belediye") !== -1 || sName.indexOf("asir") !== -1 || sName.indexOf("bursa") !== -1;
        }
        if (c === "linux") {
            return sCat.indexOf("linux") !== -1 || sName.indexOf("linux") !== -1 || sDomain.indexOf("phoronix") !== -1 || sDomain.indexOf("gamingonlinux") !== -1 || sDomain.indexOf("boilingsteam") !== -1;
        }
        if (c === "oyun" || c === "gaming") {
            return sCat.indexOf("oyun") !== -1 || sCat.indexOf("game") !== -1 || sCat.indexOf("gaming") !== -1 || sName.indexOf("game") !== -1 || sName.indexOf("oyun") !== -1 || sDomain.indexOf("steam") !== -1 || sDomain.indexOf("gamingonlinux") !== -1 || sDomain.indexOf("boilingsteam") !== -1 || sDomain.indexOf("pcgamer") !== -1 || sDomain.indexOf("rockpapershotgun") !== -1;
        }
        if (c === "teknoloji") {
            return sCat.indexOf("teknoloji") !== -1 || sCat.indexOf("bilisim") !== -1 || sCat.indexOf("dijital") !== -1;
        }
        if (c === "donanim") {
            return sCat.indexOf("donan") !== -1 || sName.indexOf("donan") !== -1 || sName.indexOf("arsiv") !== -1 || sDomain.indexOf("hwp") !== -1;
        }
        if (c === "bilim") {
            return sCat.indexOf("bilim") !== -1 || sName.indexOf("evrim") !== -1;
        }
        if (c === "siber guvenlik" || c === "guvenlik" || c === "cybersecurity") {
            return sCat.indexOf("guvenlik") !== -1 || sCat.indexOf("security") !== -1 || sCat.indexOf("cyber") !== -1 || sName.indexOf("hacker") !== -1 || sDomain.indexOf("bleepingcomputer") !== -1;
        }
        if (c === "girisimcilik" || c === "girisim") {
            return sCat.indexOf("girisim") !== -1;
        }

        return sCat.indexOf(c) !== -1 || c.indexOf(sCat) !== -1;
    }

    function getFilteredSources() {
        if (!root.sources || root.sources.length === 0) return [];
        if (root.activeCategory === "Tümü") return root.sources;
        var res = [];
        for (var i = 0; i < root.sources.length; i++) {
            if (isSourceMatchingCategory(root.sources[i], root.activeCategory)) {
                res.push(root.sources[i]);
            }
        }
        return res;
    }

    function categoryLabel(cat) {
        if (!cat) return "";
        var keyMap = {
            "Tümü": "cat_all",
            "Gündem": "cat_agenda",
            "Siyaset": "cat_politics",
            "Yerel": "cat_local",
            "Teknoloji": "cat_tech",
            "Linux": "cat_linux",
            "Oyun": "cat_gaming",
            "Donanım": "cat_hardware",
            "Bilim": "cat_science",
            "Siber Güvenlik": "cat_cybersecurity",
            "Girişimcilik": "cat_startups"
        };
        var k = keyMap[cat];
        return k ? I18n.t(k) : cat;
    }

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

            // Sync/Refresh Button (with active animation)
            Rectangle {
                width: 28
                height: 28
                radius: Theme.radiusSm
                color: refreshMouse.containsMouse ? Theme.bgCardHover : "transparent"

                Text {
                    id: refreshIcon
                    anchors.centerIn: parent
                    text: Theme.iconRefresh
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: root.isSyncing ? Theme.accentCyan : (refreshMouse.containsMouse ? Theme.accent : Theme.textMuted)

                    RotationAnimation on rotation {
                        running: root.isSyncing
                        loops: Animation.Infinite
                        from: 0
                        to: 360
                        duration: 800
                    }
                }

                MouseArea {
                    id: refreshMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    enabled: !root.isSyncing
                    onClicked: root.refreshRequested()
                }
            }

            // Info / About Button
            Rectangle {
                width: 28
                height: 28
                radius: Theme.radiusSm
                color: infoMouse.containsMouse ? Theme.bgCardHover : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: Theme.iconInfo
                    font.family: Theme.iconFont
                    font.pixelSize: 13
                    color: infoMouse.containsMouse ? Theme.accentCyan : Theme.textMuted
                }

                MouseArea {
                    id: infoMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.openInfoModal()
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
                    text: I18n.t("ai_add_source")
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
            text: I18n.t("categories")
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
                model: ["Tümü", "Gündem", "Siyaset", "Yerel", "Teknoloji", "Linux", "Oyun", "Donanım", "Bilim", "Girişimcilik"]

                Rectangle {
                    width: catText.implicitWidth + 16
                    height: 24
                    radius: Theme.radiusSm
                    color: root.activeCategory === modelData ? Theme.accent : (catMouse.containsMouse ? Theme.bgCardHover : Theme.bgSurface)

                    Text {
                        id: catText
                        anchors.centerIn: parent
                        text: root.categoryLabel(modelData)
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
                            root.selectedSourceId = "";
                            root.sourceSelected("");
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
                text: I18n.t("sources") + " (" + root.getFilteredSources().length + ")"
                font.family: Theme.fontFamily
                font.pixelSize: 10
                font.bold: true
                color: Theme.textDim
                Layout.fillWidth: true
            }

            Text {
                visible: root.unreadCount > 0
                text: root.unreadCount + " " + I18n.t("new_count")
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
                model: root.getFilteredSources()
                spacing: 4

                delegate: Rectangle {
                    id: srcItemRect
                    required property var modelData
                    width: sourceListView.width
                    height: 32
                    radius: Theme.radiusSm
                    color: root.selectedSourceId === modelData.id ? Theme.bgCard : (srcMouse.containsMouse || delBtnMouse.containsMouse ? Theme.bgCardHover : "transparent")

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

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 8
                        anchors.rightMargin: 6
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
                            visible: !srcMouse.containsMouse && !delBtnMouse.containsMouse
                            text: modelData.category
                            font.family: Theme.fontFamily
                            font.pixelSize: 9
                            color: Theme.textDim
                        }

                        // Remove source button (visible when row is hovered)
                        Rectangle {
                            id: delBtn
                            visible: srcMouse.containsMouse || delBtnMouse.containsMouse
                            width: 22
                            height: 22
                            radius: 4
                            color: delBtnMouse.containsMouse ? Theme.accentRed : Theme.bgSurface
                            border.color: delBtnMouse.containsMouse ? Theme.accentRed : Theme.border
                            border.width: 1

                            Text {
                                anchors.centerIn: parent
                                text: Theme.iconTrash
                                font.family: Theme.iconFont
                                font.pixelSize: 11
                                color: delBtnMouse.containsMouse ? Theme.bgDark : Theme.accentRed
                            }

                            MouseArea {
                                id: delBtnMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.removeSourceRequested(modelData.id, modelData.name);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Bottom Footer info
        Text {
            Layout.fillWidth: true
            text: I18n.t("footer_tagline")
            font.family: Theme.fontFamily
            font.pixelSize: 9
            color: Theme.textDim
            horizontalAlignment: Text.AlignHCenter
        }
    }
}

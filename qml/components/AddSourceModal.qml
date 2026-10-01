import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import "../theme"

Rectangle {
    id: root

    property bool isSearching: false
    property string statusMessage: ""
    property int currentTab: 0 // 0: AI, 1: Manual
    property string manualCategory: "Teknoloji"

    signal closeRequested()
    signal promptSubmitted(string prompt)
    signal manualSourceSubmitted(string name, string url, string category)

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
                    text: root.currentTab === 0 ? Theme.iconAi : Theme.iconRss
                    font.family: Theme.iconFont
                    font.pixelSize: 18
                    color: Theme.accent
                }

                Text {
                    text: root.currentTab === 0 ? I18n.t("modal_title") : (I18n.currentLanguage === "en" ? "Add News Source Manually" : "Manuel Haber Kaynagi Ekle")
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

            // Tab Navigation Switcher
            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Rectangle {
                    Layout.fillWidth: true
                    height: 32
                    radius: Theme.radiusSm
                    color: root.currentTab === 0 ? Theme.accent : Theme.bgSurface
                    border.color: root.currentTab === 0 ? Theme.accent : Theme.border
                    border.width: 1

                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 6

                        Text {
                            text: Theme.iconAi
                            font.family: Theme.iconFont
                            font.pixelSize: 12
                            color: root.currentTab === 0 ? Theme.bgDark : Theme.textMuted
                        }

                        Text {
                            text: I18n.t("tab_ai_add")
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            font.bold: root.currentTab === 0
                            color: root.currentTab === 0 ? Theme.bgDark : Theme.textMain
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        enabled: !root.isSearching
                        onClicked: {
                            root.currentTab = 0;
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 32
                    radius: Theme.radiusSm
                    color: root.currentTab === 1 ? Theme.accent : Theme.bgSurface
                    border.color: root.currentTab === 1 ? Theme.accent : Theme.border
                    border.width: 1

                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 6

                        Text {
                            text: Theme.iconRss
                            font.family: Theme.iconFont
                            font.pixelSize: 12
                            color: root.currentTab === 1 ? Theme.bgDark : Theme.textMuted
                        }

                        Text {
                            text: I18n.t("tab_manual_add")
                            font.family: Theme.fontFamily
                            font.pixelSize: 11
                            font.bold: root.currentTab === 1
                            color: root.currentTab === 1 ? Theme.bgDark : Theme.textMain
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        enabled: !root.isSearching
                        onClicked: {
                            root.currentTab = 1;
                        }
                    }
                }
            }

            // === TAB 0: AI AUTO-DISCOVER ===
            ColumnLayout {
                visible: root.currentTab === 0
                Layout.fillWidth: true
                spacing: 12

                Text {
                    text: I18n.t("modal_sub")
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
                                text: I18n.t("modal_input_placeholder")
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
                    text: (I18n.currentLanguage === "en") ? "QUICK EXAMPLES (Click to select):" : "HAZIR ORNEKLER (Tiklayarak secin):"
                    font.family: Theme.fontFamily
                    font.pixelSize: 9
                    font.bold: true
                    color: Theme.textDim
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 6

                    Repeater {
                        model: (I18n.currentLanguage === "en") ? [
                            "Add top US technology news and AI blogs",
                            "Add Linux, kernel and open-source tech sources",
                            "Add Austin and Texas local news",
                            "Add science and space exploration sites",
                            "Add global economics and market news",
                            "Add hardware and PC enthusiast media",
                            "Add startup and venture capital portals"
                        ] : [
                            "Turkiye gundemi ve siyasi haber kaynaklarini ekle",
                            "Guvenilir genel haber gazeteleri ve ajanslarini ekle",
                            "Istanbul, Ankara ve yerel sehir haberlerini ekle",
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
            }

            // === TAB 1: MANUAL ADD ===
            ColumnLayout {
                visible: root.currentTab === 1
                Layout.fillWidth: true
                spacing: 12

                Text {
                    text: (I18n.currentLanguage === "en") ? "Directly add a website address or RSS feed link. The engine will discover and verify the feed automatically." : "Bir site adresi veya RSS linki girin. QuickNews motoru akisi otomatik kesfedecek ve dogrulayacaktir."
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    color: Theme.textMuted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }

                // Name input
                Text {
                    text: I18n.t("manual_name_label")
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.bold: true
                    color: Theme.textMuted
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 38
                    radius: Theme.radiusSm
                    color: Theme.bgSurface
                    border.color: manualNameInput.activeFocus ? Theme.accent : Theme.border
                    border.width: 1

                    TextInput {
                        id: manualNameInput
                        anchors.fill: parent
                        anchors.margins: 10
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        color: Theme.textMain
                        selectByMouse: true
                        clip: true
                        enabled: !root.isSearching

                        Text {
                            anchors.fill: parent
                            text: (I18n.currentLanguage === "en") ? "e.g. AnandTech" : "Orn: Webtekno, DonanimHaber..."
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            color: Theme.textDim
                            visible: !manualNameInput.text && !manualNameInput.activeFocus
                        }
                    }
                }

                // URL input
                Text {
                    text: I18n.t("manual_url_label")
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.bold: true
                    color: Theme.textMuted
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 38
                    radius: Theme.radiusSm
                    color: Theme.bgSurface
                    border.color: manualUrlInput.activeFocus ? Theme.accent : Theme.border
                    border.width: 1

                    TextInput {
                        id: manualUrlInput
                        anchors.fill: parent
                        anchors.margins: 10
                        font.family: Theme.fontFamily
                        font.pixelSize: 12
                        color: Theme.textMain
                        selectByMouse: true
                        clip: true
                        enabled: !root.isSearching

                        Text {
                            anchors.fill: parent
                            text: "https://webtekno.com (veya feed url)"
                            font.family: Theme.fontFamily
                            font.pixelSize: 12
                            color: Theme.textDim
                            visible: !manualUrlInput.text && !manualUrlInput.activeFocus
                        }
                    }
                }

                // Category selector
                Text {
                    text: I18n.t("manual_category_label")
                    font.family: Theme.fontFamily
                    font.pixelSize: 10
                    font.bold: true
                    color: Theme.textMuted
                }

                Flow {
                    Layout.fillWidth: true
                    spacing: 6

                    Repeater {
                        model: [
                            "Teknoloji", "Gündem", "Siyaset", "Yerel",
                            "Linux", "Oyun", "Donanım", "Bilim", "Girişimcilik"
                        ]

                        Rectangle {
                            width: catText.implicitWidth + 16
                            height: 26
                            radius: Theme.radiusSm
                            color: root.manualCategory === modelData ? Theme.accent : Theme.bgSurface
                            border.color: root.manualCategory === modelData ? Theme.accent : Theme.border
                            border.width: 1

                            Text {
                                id: catText
                                anchors.centerIn: parent
                                text: modelData
                                font.family: Theme.fontFamily
                                font.pixelSize: 10
                                font.bold: root.manualCategory === modelData
                                color: root.manualCategory === modelData ? Theme.bgDark : Theme.textMain
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.manualCategory = modelData;
                                }
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
                        text: root.isSearching ? Theme.iconRefresh : (root.currentTab === 0 ? Theme.iconAi : Theme.iconPlus)
                        font.family: Theme.iconFont
                        font.pixelSize: 13
                        color: root.isSearching ? Theme.textMuted : Theme.bgDark
                    }

                    Text {
                        text: root.isSearching ? I18n.t("modal_searching") : (root.currentTab === 0 ? I18n.t("modal_submit") : I18n.t("manual_submit_btn"))
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
                        if (root.currentTab === 0) {
                            if (promptInput.text.trim().length > 0) {
                                root.promptSubmitted(promptInput.text.trim());
                            }
                        } else {
                            var n = manualNameInput.text.trim();
                            var u = manualUrlInput.text.trim();
                            if (u.length > 0) {
                                if (n.length === 0) {
                                    try {
                                        var parsed = u.replace(/^https?:\/\//i, "").split("/")[0];
                                        n = parsed;
                                    } catch(e) {
                                        n = "Yeni Kaynak";
                                    }
                                }
                                root.manualSourceSubmitted(n, u, root.manualCategory);
                            }
                        }
                    }
                }
            }
        }
    }
}

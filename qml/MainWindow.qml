import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import "theme"
import "components"

Rectangle {
    id: root

    property var sourcesList: []
    property var articlesList: []
    property var savedArticlesList: []
    property var selectedArticle: null
    property var fullCleanArticle: null
    property var aiSummaryData: null
    property bool isLoadingContent: false
    property bool isLoadingAi: false
    property bool isSyncing: false
    property bool isAddingPrompt: false
    property string promptStatus: ""
    property bool showAddModal: false
    property bool isZenMode: false
    property string articlesBuffer: ""
    property string readBuffer: ""
    property string sourcesBuffer: ""
    property string savedBuffer: ""
    property string summaryBuffer: ""
    property string promptBuffer: ""

    color: Theme.bgDark
    focus: true

    readonly property string engineBin: {
        var home = Quickshell.env("HOME");
        return (home ? home : "") + "/.local/bin/quicknews-engine";
    }

    readonly property bool isCurrentArticleSaved: {
        if (!root.selectedArticle || !root.savedArticlesList) return false;
        for (var i = 0; i < root.savedArticlesList.length; i++) {
            if (root.savedArticlesList[i].id === root.selectedArticle.id ||
                root.savedArticlesList[i].link === root.selectedArticle.link) {
                return true;
            }
        }
        return false;
    }

    Component.onCompleted: {
        loadSources();
        loadArticles();
        loadSavedArticles();
    }

    function loadSources() {
        root.sourcesBuffer = "";
        sourcesProc.command = [root.engineBin, "sources", "--json"];
        sourcesProc.running = true;
    }

    function loadArticles() {
        root.articlesBuffer = "";
        articlesProc.command = [root.engineBin, "list", "--json"];
        articlesProc.running = true;
    }

    function loadSavedArticles() {
        root.savedBuffer = "";
        savedProc.command = [root.engineBin, "saved", "--json"];
        savedProc.running = true;
    }

    function syncFeeds() {
        if (root.isSyncing) return;
        root.isSyncing = true;
        syncProc.command = [root.engineBin, "sync"];
        syncProc.running = true;
    }

    function loadArticleContent(article) {
        root.selectedArticle = article;
        root.fullCleanArticle = null;
        root.aiSummaryData = null;
        root.isLoadingContent = true;
        root.readBuffer = "";

        // Mark as read in engine
        markReadProc.command = [root.engineBin, "mark-read", article.id];
        markReadProc.running = true;

        // Read clean text
        readProc.command = [root.engineBin, "read", article.link, "--json"];
        readProc.running = true;
    }

    function toggleSaveArticle(idOrLink) {
        if (!idOrLink) return;
        toggleSaveProc.command = [root.engineBin, "toggle-save", idOrLink];
        toggleSaveProc.running = true;
    }

    function exportArticle(url) {
        if (!url) return;
        exportProc.command = [root.engineBin, "export", url, "--json"];
        exportProc.running = true;
    }

    function requestAiSummary(url) {
        root.isLoadingAi = true;
        root.summaryBuffer = "";
        summaryProc.command = [root.engineBin, "summarize", url, "--json"];
        summaryProc.running = true;
    }

    function submitPromptAdd(promptText) {
        root.isAddingPrompt = true;
        root.promptBuffer = "";
        root.promptStatus = "Yapay zeka kaynaklari analiz ediyor ve RSS akislarini dogruluyor...";
        addPromptProc.command = [root.engineBin, "add-prompt", promptText, "--json"];
        addPromptProc.running = true;
    }

    // Keyboard navigation and shortcuts
    Keys.onPressed: function(event) {
        if (root.showAddModal) {
            if (event.key === Qt.Key_Escape) {
                root.showAddModal = false;
                event.accepted = true;
            }
            return;
        }

        // Global shortcuts
        if (event.key === Qt.Key_J || event.key === Qt.Key_Down) {
            headlineList.selectNext();
            event.accepted = true;
        } else if (event.key === Qt.Key_K || event.key === Qt.Key_Up) {
            headlineList.selectPrev();
            event.accepted = true;
        } else if (event.key === Qt.Key_F) {
            root.isZenMode = !root.isZenMode;
            event.accepted = true;
        } else if (event.key === Qt.Key_S) {
            if (root.selectedArticle) {
                root.toggleSaveArticle(root.selectedArticle.id);
            }
            event.accepted = true;
        } else if (event.key === Qt.Key_R) {
            root.syncFeeds();
            event.accepted = true;
        } else if (event.key === Qt.Key_Slash) {
            headlineList.focusSearch();
            event.accepted = true;
        } else if (event.key === Qt.Key_Escape) {
            if (root.isZenMode) {
                root.isZenMode = false;
                event.accepted = true;
            }
        }
    }

    // Main 3-Pane Layout
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Left: Source and Category Sidebar
        SourceSidebar {
            id: sidebar
            visible: !root.isZenMode
            Layout.fillHeight: true
            Layout.preferredWidth: root.isZenMode ? 0 : 250
            Layout.minimumWidth: root.isZenMode ? 0 : 220
            sources: root.sourcesList
            isSyncing: root.isSyncing
            unreadCount: {
                var c = 0;
                for (var i = 0; i < root.articlesList.length; i++) {
                    if (!root.articlesList[i].is_read) c++;
                }
                return c;
            }
            onCategorySelected: function(cat) {
                headlineList.activeCategory = cat;
            }
            onSourceSelected: function(srcId) {
                headlineList.activeSourceId = srcId;
            }
            onOpenAddModal: {
                root.showAddModal = true;
                root.promptStatus = "";
            }
            onRefreshRequested: {
                root.syncFeeds();
            }
        }

        // Center: Article Headlines List
        HeadlineList {
            id: headlineList
            visible: !root.isZenMode
            Layout.fillHeight: true
            Layout.preferredWidth: root.isZenMode ? 0 : 380
            Layout.minimumWidth: root.isZenMode ? 0 : 320
            articles: root.articlesList
            savedArticles: root.savedArticlesList
            selectedArticleId: root.selectedArticle ? root.selectedArticle.id : ""
            onArticleSelected: function(art) {
                root.loadArticleContent(art);
            }
        }

        // Right: Clean Distraction-Free Article Reader
        ArticleReader {
            id: reader
            Layout.fillHeight: true
            Layout.fillWidth: true
            currentArticle: root.selectedArticle
            fullCleanArticle: root.fullCleanArticle
            aiSummary: root.aiSummaryData
            isLoadingContent: root.isLoadingContent
            isLoadingAi: root.isLoadingAi
            isSaved: root.isCurrentArticleSaved
            isZenMode: root.isZenMode
            onSummarizeRequested: function(url) {
                root.requestAiSummary(url);
            }
            onOpenExternalRequested: function(url) {
                Qt.openUrlExternally(url);
            }
            onToggleSaveRequested: function(url) {
                root.toggleSaveArticle(url);
            }
            onToggleZenRequested: {
                root.isZenMode = !root.isZenMode;
            }
            onExportRequested: function(url) {
                root.exportArticle(url);
            }
        }
    }

    // Add Source Natural Language Modal
    AddSourceModal {
        anchors.fill: parent
        visible: root.showAddModal
        isSearching: root.isAddingPrompt
        statusMessage: root.promptStatus
        onCloseRequested: {
            root.showAddModal = false;
        }
        onPromptSubmitted: function(p) {
            root.submitPromptAdd(p);
        }
    }

    // Background Engine Processes
    Process {
        id: sourcesProc
        stdout: SplitParser {
            onRead: function(data) {
                root.sourcesBuffer += data;
            }
        }
        onExited: function(exitCode) {
            try {
                if (root.sourcesBuffer.trim().length > 0) {
                    root.sourcesList = JSON.parse(root.sourcesBuffer);
                }
            } catch(e) {}
            root.sourcesBuffer = "";
        }
    }

    Process {
        id: articlesProc
        stdout: SplitParser {
            onRead: function(data) {
                root.articlesBuffer += data;
            }
        }
        onExited: function(exitCode) {
            try {
                if (root.articlesBuffer.trim().length > 0) {
                    root.articlesList = JSON.parse(root.articlesBuffer);
                }
            } catch(e) {}
            root.articlesBuffer = "";
        }
    }

    Process {
        id: savedProc
        stdout: SplitParser {
            onRead: function(data) {
                root.savedBuffer += data;
            }
        }
        onExited: function(exitCode) {
            try {
                if (root.savedBuffer.trim().length > 0) {
                    root.savedArticlesList = JSON.parse(root.savedBuffer);
                }
            } catch(e) {}
            root.savedBuffer = "";
        }
    }

    Process {
        id: syncProc
        onExited: function(exitCode) {
            root.isSyncing = false;
            root.loadArticles();
            root.loadSources();
            root.loadSavedArticles();
        }
    }

    Process {
        id: toggleSaveProc
        onExited: function(exitCode) {
            root.loadSavedArticles();
        }
    }

    Process {
        id: exportProc
        stdout: SplitParser {
            onRead: function(data) {
                try {
                    var res = JSON.parse(data);
                } catch(e) {}
            }
        }
    }

    Process {
        id: readProc
        stdout: SplitParser {
            onRead: function(data) {
                root.readBuffer += data;
            }
        }
        onExited: function(exitCode) {
            try {
                if (root.readBuffer.trim().length > 0) {
                    root.fullCleanArticle = JSON.parse(root.readBuffer);
                }
            } catch(e) {}
            root.readBuffer = "";
            root.isLoadingContent = false;
        }
    }

    Process {
        id: summaryProc
        stdout: SplitParser {
            onRead: function(data) {
                root.summaryBuffer += data;
            }
        }
        onExited: function(exitCode) {
            try {
                if (root.summaryBuffer.trim().length > 0) {
                    root.aiSummaryData = JSON.parse(root.summaryBuffer);
                }
            } catch(e) {}
            root.summaryBuffer = "";
            root.isLoadingAi = false;
        }
    }

    Process {
        id: addPromptProc
        stdout: SplitParser {
            onRead: function(data) {
                root.promptBuffer += data;
            }
        }
        onExited: function(exitCode) {
            root.isAddingPrompt = false;
            try {
                if (root.promptBuffer.trim().length > 0) {
                    var added = JSON.parse(root.promptBuffer);
                    root.promptStatus = "Basariyla " + added.length + " yeni kaynak eklendi!";
                    root.loadSources();
                    root.syncFeeds();
                } else {
                    root.promptStatus = "Kaynaklar kontrol edildi.";
                    root.loadSources();
                }
            } catch(e) {
                root.promptStatus = "Kaynaklar eklendi.";
                root.loadSources();
            }
            root.promptBuffer = "";
        }
    }

    Process {
        id: markReadProc
    }
}

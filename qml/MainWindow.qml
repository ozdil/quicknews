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
    property var selectedArticle: null
    property var fullCleanArticle: null
    property var aiSummaryData: null
    property bool isLoadingContent: false
    property bool isLoadingAi: false
    property bool isSyncing: false
    property bool isAddingPrompt: false
    property string promptStatus: ""
    property bool showAddModal: false

    color: Theme.bgDark

    readonly property string engineBin: {
        var localBin = Quickshell.env("HOME") + "/.local/bin/quicknews-engine";
        var cwdBin = Quickshell.env("PWD") + "/target/release/quicknews-engine";
        return cwdBin;
    }

    Component.onCompleted: {
        loadSources();
        loadArticles();
    }

    function loadSources() {
        sourcesProc.command = [root.engineBin, "sources", "--json"];
        sourcesProc.running = true;
    }

    function loadArticles() {
        articlesProc.command = [root.engineBin, "list", "--json"];
        articlesProc.running = true;
    }

    function syncFeeds() {
        root.isSyncing = true;
        syncProc.command = [root.engineBin, "sync", "--json"];
        syncProc.running = true;
    }

    function loadArticleContent(article) {
        root.selectedArticle = article;
        root.fullCleanArticle = null;
        root.aiSummaryData = null;
        root.isLoadingContent = true;

        // Mark as read in engine
        markReadProc.command = [root.engineBin, "mark-read", article.id];
        markReadProc.running = true;

        // Read clean text
        readProc.command = [root.engineBin, "read", article.link, "--json"];
        readProc.running = true;
    }

    function requestAiSummary(url) {
        root.isLoadingAi = true;
        summaryProc.command = [root.engineBin, "summarize", url, "--json"];
        summaryProc.running = true;
    }

    function submitPromptAdd(promptText) {
        root.isAddingPrompt = true;
        root.promptStatus = "Yapay zeka kaynaklari analiz ediyor ve RSS akislarini dogruluyor...";
        addPromptProc.command = [root.engineBin, "add-prompt", promptText, "--json"];
        addPromptProc.running = true;
    }

    // Main 3-Pane Layout
    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Left: Source and Category Sidebar
        SourceSidebar {
            id: sidebar
            Layout.fillHeight: true
            Layout.preferredWidth: 250
            Layout.minimumWidth: 220
            sources: root.sourcesList
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
            Layout.fillHeight: true
            Layout.preferredWidth: 380
            Layout.minimumWidth: 320
            articles: root.articlesList
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
            onSummarizeRequested: function(url) {
                root.requestAiSummary(url);
            }
            onOpenExternalRequested: function(url) {
                Qt.openUrlExternally(url);
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
                try {
                    root.sourcesList = JSON.parse(data);
                } catch(e) {}
            }
        }
    }

    Process {
        id: articlesProc
        stdout: SplitParser {
            onRead: function(data) {
                try {
                    root.articlesList = JSON.parse(data);
                } catch(e) {}
            }
        }
    }

    Process {
        id: syncProc
        stdout: SplitParser {
            onRead: function(data) {
                try {
                    root.articlesList = JSON.parse(data);
                } catch(e) {}
                root.isSyncing = false;
            }
        }
    }

    Process {
        id: readProc
        stdout: SplitParser {
            onRead: function(data) {
                try {
                    root.fullCleanArticle = JSON.parse(data);
                } catch(e) {}
                root.isLoadingContent = false;
            }
        }
    }

    Process {
        id: summaryProc
        stdout: SplitParser {
            onRead: function(data) {
                try {
                    root.aiSummaryData = JSON.parse(data);
                } catch(e) {}
                root.isLoadingAi = false;
            }
        }
    }

    Process {
        id: addPromptProc
        stdout: SplitParser {
            onRead: function(data) {
                root.isAddingPrompt = false;
                try {
                    var added = JSON.parse(data);
                    root.promptStatus = "Basariyla " + added.length + " yeni kaynak eklendi!";
                    root.loadSources();
                    root.syncFeeds();
                } catch(e) {
                    root.promptStatus = "Kaynaklar eklendi.";
                    root.loadSources();
                }
            }
        }
    }

    Process {
        id: markReadProc
    }
}

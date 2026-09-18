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

        // Read clean text (Keep article visible in unread list until explicitly dismissed)
        readProc.command = [root.engineBin, "read", "--", article.link, "--json"];
        readProc.running = true;
    }

    function dismissArticle(idOrLink) {
        if (!idOrLink) return;

        // 1. Mark as read in frontend model so it drops from 'Okunmamış' list
        for (var i = 0; i < root.articlesList.length; i++) {
            if (root.articlesList[i].id === idOrLink || root.articlesList[i].link === idOrLink) {
                root.articlesList[i].is_read = true;
                break;
            }
        }
        root.articlesList = root.articlesList.slice();

        // 2. Clear right pane reader content immediately (asıl içerik ve sol başlık birlikte uçar!)
        if (root.selectedArticle && (root.selectedArticle.id === idOrLink || root.selectedArticle.link === idOrLink)) {
            root.selectedArticle = null;
            root.fullCleanArticle = null;
            root.aiSummaryData = null;
            root.isLoadingContent = false;
        }

        // 3. Mark as read in engine
        markReadProc.command = [root.engineBin, "mark-read", "--", idOrLink];
        markReadProc.running = true;
    }

    function toggleReadArticle(idOrLink) {
        if (!idOrLink) return;
        var found = false;
        var newState = false;
        for (var i = 0; i < root.articlesList.length; i++) {
            if (root.articlesList[i].id === idOrLink || root.articlesList[i].link === idOrLink) {
                root.articlesList[i].is_read = !root.articlesList[i].is_read;
                newState = root.articlesList[i].is_read;
                found = true;
                break;
            }
        }
        if (found) {
            root.articlesList = root.articlesList.slice();
            // If the dismissed article was open in the reader and became read, dismiss reader content
            if (newState && root.selectedArticle && (root.selectedArticle.id === idOrLink || root.selectedArticle.link === idOrLink)) {
                root.selectedArticle = null;
                root.fullCleanArticle = null;
                root.aiSummaryData = null;
                root.isLoadingContent = false;
            } else if (root.selectedArticle && (root.selectedArticle.id === idOrLink || root.selectedArticle.link === idOrLink)) {
                root.selectedArticle.is_read = newState;
            }
        }
        toggleReadProc.command = [root.engineBin, "toggle-read", "--", idOrLink];
        toggleReadProc.running = true;
    }

    function toggleSaveArticle(idOrLink) {
        if (!idOrLink) return;
        toggleSaveProc.command = [root.engineBin, "toggle-save", "--", idOrLink];
        toggleSaveProc.running = true;
    }

    function exportArticle(url) {
        if (!url) return;
        exportProc.command = [root.engineBin, "export", "--", url, "--json"];
        exportProc.running = true;
    }

    function requestAiSummary(url) {
        root.isLoadingAi = true;
        root.summaryBuffer = "";
        summaryProc.command = [root.engineBin, "summarize", "--", url, "--json"];
        summaryProc.running = true;
    }

    function submitPromptAdd(promptText) {
        root.isAddingPrompt = true;
        root.promptBuffer = "";
        root.promptStatus = I18n.t("modal_searching");
        addPromptProc.command = [root.engineBin, "add-prompt", "--", promptText, "--json"];
        addPromptProc.running = true;
    }

    function removeSource(idOrDomain) {
        if (!idOrDomain) return;

        // 1. Optimistic removal from frontend lists
        var updatedSources = [];
        for (var i = 0; i < root.sourcesList.length; i++) {
            if (root.sourcesList[i].id !== idOrDomain && root.sourcesList[i].domain !== idOrDomain) {
                updatedSources.push(root.sourcesList[i]);
            }
        }
        root.sourcesList = updatedSources;

        var updatedArticles = [];
        for (var j = 0; j < root.articlesList.length; j++) {
            if (root.articlesList[j].source_id !== idOrDomain && root.articlesList[j].source_name !== idOrDomain) {
                updatedArticles.push(root.articlesList[j]);
            }
        }
        root.articlesList = updatedArticles;

        if (headlineList.activeSourceId === idOrDomain) {
            headlineList.activeSourceId = "";
        }

        if (root.selectedArticle && (root.selectedArticle.source_id === idOrDomain || root.selectedArticle.source_name === idOrDomain)) {
            root.selectedArticle = null;
            root.fullCleanArticle = null;
            root.aiSummaryData = null;
            root.isLoadingContent = false;
        }

        // 2. Call backend removal process
        removeSourceProc.command = [root.engineBin, "remove-source", "--", idOrDomain];
        removeSourceProc.running = true;
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

        // When search input is focused, let normal text entry pass without hijacking shortcuts
        if (headlineList.isSearchFocused) {
            if (event.key === Qt.Key_Escape) {
                headlineList.clearSearch();
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

    function applyTagCategory(tag) {
        if (!tag) return;
        var mappedCat = tag;
        var t = norm(tag);
        if (t.indexOf("gundem") !== -1 || t.indexOf("genel") !== -1) {
            mappedCat = "Gündem";
        } else if (t.indexOf("siyaset") !== -1 || t.indexOf("politika") !== -1) {
            mappedCat = "Siyaset";
        } else if (t.indexOf("yerel") !== -1 || t.indexOf("belediye") !== -1) {
            mappedCat = "Yerel";
        } else if (t.indexOf("linux") !== -1) {
            mappedCat = "Linux";
        } else if (t.indexOf("teknoloji") !== -1 || t.indexOf("yapay zeka") !== -1) {
            mappedCat = "Teknoloji";
        } else if (t.indexOf("donan") !== -1) {
            mappedCat = "Donanım";
        } else if (t.indexOf("bilim") !== -1) {
            mappedCat = "Bilim";
        } else if (t.indexOf("girisim") !== -1) {
            mappedCat = "Girişimcilik";
        }
        sidebar.activeCategory = mappedCat;
        sidebar.selectedSourceId = "";
        headlineList.activeCategory = mappedCat;
        headlineList.activeSourceId = "";
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
            onRemoveSourceRequested: function(sourceId, sourceName) {
                root.removeSource(sourceId);
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
            onToggleReadRequested: function(artId) {
                root.toggleReadArticle(artId);
            }
            onTagSelected: function(tag) {
                root.applyTagCategory(tag);
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
                if (!url) return;
                var u = String(url).trim().toLowerCase();
                if (u.indexOf("http://") === 0 || u.indexOf("https://") === 0) {
                    Qt.openUrlExternally(url);
                }
            }
            onToggleSaveRequested: function(url) {
                root.toggleSaveArticle(url);
            }
            onToggleReadRequested: function(artId) {
                root.toggleReadArticle(artId);
            }
            onDismissArticleRequested: function(artId) {
                root.dismissArticle(artId);
            }
            onToggleZenRequested: {
                root.isZenMode = !root.isZenMode;
            }
            onExportRequested: function(url) {
                root.exportArticle(url);
            }
            onTagSelected: function(tag) {
                root.applyTagCategory(tag);
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
                    if (Array.isArray(added) && added.length > 0) {
                        root.promptStatus = (I18n.currentLanguage === "en") ? ("Successfully added " + added.length + " new sources!") : ("Basariyla " + added.length + " yeni kaynak eklendi!");
                        root.loadSources();
                        root.syncFeeds();
                    } else {
                        root.promptStatus = (I18n.currentLanguage === "en") ? "All matching sources are already in your list or no active feeds found." : "Eslenen kaynaklar zaten listenizde ekli veya aktif akis bulunamadi.";
                        root.loadSources();
                    }
                } else {
                    root.promptStatus = I18n.t("modal_controlled");
                    root.loadSources();
                }
            } catch(e) {
                root.promptStatus = (I18n.currentLanguage === "en") ? "Sources updated." : "Kaynaklar guncellendi.";
                root.loadSources();
            }
            root.promptBuffer = "";
        }
    }

    Process {
        id: markReadProc
    }

    Process {
        id: toggleReadProc
    }

    Process {
        id: removeSourceProc
        onExited: function(exitCode) {
            root.loadSources();
            root.loadArticles();
        }
    }
}

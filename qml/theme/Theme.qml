pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

QtObject {
    id: root

    // Active Omarchy Theme Information
    property string themeName: "default"
    property bool isDarkTheme: true

    // Dynamic Color Palette (Reactive bindings update all UI elements live)
    property color bgDark: "#13141c"
    property color bgBase: "#16161e"
    property color bgSurface: "#1a1b26"
    property color bgCard: "#202330"
    property color bgCardHover: "#282c3f"
    property color border: "#2f354a"
    property color borderLight: "#414868"

    property color textMain: "#c0caf5"
    property color textMuted: "#7982a9"
    property color textDim: "#545c7e"

    property color accent: "#7aa2f7"
    property color accentHover: "#89b4fa"
    property color accentPurple: "#bb9af7"
    property color accentGreen: "#9ece6a"
    property color accentOrange: "#ff9e64"
    property color accentCyan: "#7dcfff"
    property color accentRed: "#f7768e"

    // UI Radii and Spacings
    readonly property int radiusSm: 6
    readonly property int radiusMd: 10
    readonly property int radiusLg: 14

    // Typography: Strict Omarchy Standard
    readonly property string fontFamily: "JetBrainsMono Nerd Font, JetBrains Mono, monospace"
    readonly property string monoFont: "JetBrainsMono Nerd Font, JetBrains Mono, monospace"
    readonly property string iconFont: "JetBrainsMono Nerd Font, Font Awesome 7 Free Solid, monospace"

    // Themeable Monochrome Icons (Unicode Font Glyphs, zero emoji)
    readonly property string iconNews: "\uf1ea"
    readonly property string iconFeed: "\uf09e"
    readonly property string iconRefresh: "\uf021"
    readonly property string iconSearch: "\uf002"
    readonly property string iconAi: "\uf0e7"
    readonly property string iconClock: "\uf017"
    readonly property string iconCheck: "\uf00c"
    readonly property string iconPlus: "\uf067"
    readonly property string iconTrash: "\uf1f8"
    readonly property string iconTimes: "\uf00d"
    readonly property string iconExpand: "\uf065"
    readonly property string iconCompress: "\uf066"
    readonly property string iconSliders: "\uf1de"
    readonly property string iconBook: "\uf02d"
    readonly property string iconExternalLink: "\uf08e"
    readonly property string iconFontPlus: "\uf034"
    readonly property string iconFontMinus: "\uf035"
    readonly property string iconBookmark: "\uf02e"
    readonly property string iconBookmarkOutline: "\uf097"
    readonly property string iconExport: "\uf019"
    readonly property string iconFilter: "\uf0b0"
    readonly property string iconCheckCircle: "\uf058"
    readonly property string iconCircleOutline: "\uf111"
    readonly property string iconCircleDot: "\uf192"
    readonly property string iconInfo: "\uf05a"
    readonly property string iconShield: "\uf3ed"
    readonly property string iconCoffee: "\uf0f4"

    // Filesystem Paths for Omarchy System Theme
    readonly property string homeDir: Quickshell.env("HOME")
    readonly property string omarchyStateDir: homeDir + "/.local/state/omarchy/current"

    property string lastLoadedRaw: ""

    function loadColors(raw) {
        if (!raw || raw.trim().length === 0 || raw === lastLoadedRaw) return;
        lastLoadedRaw = raw;

        var dict = {};
        var lines = String(raw).split("\n");
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i].trim();
            if (!line || line.charAt(0) === '#') continue;
            var match = line.match(/^([A-Za-z0-9_-]+)\s*=\s*["']?([^"'\r\n]+?)["']?\s*(?:#.*)?$/);
            if (match) {
                dict[match[1].toLowerCase()] = match[2].trim();
            }
        }

        var mode = dict["mode"] || "dark";
        root.isDarkTheme = (mode !== "light");

        // Foundational colors
        var base = dict["background"] || dict["bg"] || (root.isDarkTheme ? "#16161e" : "#f5f6f9");
        var fg = dict["foreground"] || dict["fg"] || (root.isDarkTheme ? "#c0caf5" : "#1a1b26");
        var acc = dict["accent"] || dict["color4"] || dict["color6"] || "#7aa2f7";
        var sel = dict["selection"] || dict["selection_background"] || "";
        var mut = dict["muted"] || dict["color8"] || "";

        root.bgBase = base;

        if (root.isDarkTheme) {
            root.bgDark = dict["dark_background"] || dict["darker_background"] || dict["color0"] || Qt.darker(base, 1.3);
            root.bgSurface = dict["lighter_background"] || (sel ? sel : Qt.lighter(base, 1.35));
            root.bgCard = (sel && sel !== base) ? sel : Qt.lighter(base, 1.6);
            root.bgCardHover = Qt.lighter(root.bgCard, 1.2);
            root.border = mut ? mut : Qt.rgba(fg.r, fg.g, fg.b, 0.18);
            root.borderLight = acc ? Qt.rgba(acc.r, acc.g, acc.b, 0.4) : Qt.rgba(fg.r, fg.g, fg.b, 0.28);
            root.textMain = dict["bright_foreground"] || fg;
            root.textMuted = dict["light_foreground"] || mut || Qt.rgba(fg.r, fg.g, fg.b, 0.7);
            root.textDim = dict["dark_foreground"] || dict["color8"] || Qt.rgba(fg.r, fg.g, fg.b, 0.45);
        } else {
            root.bgDark = dict["dark_background"] || Qt.darker(base, 1.08);
            root.bgSurface = dict["lighter_background"] || Qt.lighter(base, 1.02);
            root.bgCard = (sel && sel !== base) ? sel : Qt.darker(base, 1.05);
            root.bgCardHover = Qt.darker(root.bgCard, 1.06);
            root.border = mut ? mut : Qt.rgba(fg.r, fg.g, fg.b, 0.18);
            root.borderLight = acc ? Qt.rgba(acc.r, acc.g, acc.b, 0.4) : Qt.rgba(fg.r, fg.g, fg.b, 0.3);
            root.textMain = dict["bright_foreground"] || fg;
            root.textMuted = dict["light_foreground"] || mut || Qt.rgba(fg.r, fg.g, fg.b, 0.7);
            root.textDim = dict["dark_foreground"] || dict["color8"] || Qt.rgba(fg.r, fg.g, fg.b, 0.45);
        }

        root.accent = acc;
        root.accentHover = dict["bright_cyan"] || dict["bright_blue"] || Qt.lighter(acc, 1.2);
        root.accentPurple = dict["purple"] || dict["magenta"] || dict["color5"] || "#bb9af7";
        root.accentGreen = dict["bright_green"] || dict["green"] || dict["color2"] || "#9ece6a";
        root.accentOrange = dict["orange"] || dict["color9"] || dict["color1"] || "#ff9e64";
        root.accentCyan = dict["bright_cyan"] || dict["cyan"] || dict["color6"] || "#7dcfff";
        root.accentRed = dict["red"] || dict["bright_red"] || dict["color1"] || "#f7768e";
    }

    function getCategoryColor(cat) {
        if (!cat) return root.textMuted;
        var c = String(cat).toLowerCase();
        if (c.indexOf("linux") !== -1) return root.accentOrange;
        if (c.indexOf("oyun") !== -1 || c.indexOf("gaming") !== -1) return root.accentPurple;
        if (c.indexOf("guvenlik") !== -1 || c.indexOf("security") !== -1) return root.accentRed;
        if (c.indexOf("donan") !== -1) return root.accentCyan;
        if (c.indexOf("bilim") !== -1) return root.accentGreen;
        if (c.indexOf("teknoloji") !== -1 || c.indexOf("yapay") !== -1 || c.indexOf("ai") !== -1) return root.accent;
        if (c.indexOf("siyaset") !== -1 || c.indexOf("politika") !== -1) return root.textMuted;
        if (c.indexOf("gundem") !== -1) return root.accent;
        return root.accent;
    }

    // Inotify FileView on Omarchy colors.toml
    property FileView colorsFile: FileView {
        id: colorsFile
        path: root.omarchyStateDir + "/theme/colors.toml"
        watchChanges: true
        printErrors: false
        onLoaded: root.loadColors(text())
        onFileChanged: reload()
    }

    // Inotify FileView on Omarchy theme.name
    property FileView themeNameFile: FileView {
        id: themeNameFile
        path: root.omarchyStateDir + "/theme.name"
        watchChanges: true
        printErrors: false
        onLoaded: {
            var n = text().trim();
            if (n.length > 0 && n !== root.themeName) {
                root.themeName = n;
                root.lastLoadedRaw = "";
                colorsFile.reload();
            }
        }
        onFileChanged: {
            reload();
            root.lastLoadedRaw = "";
            colorsFile.reload();
        }
    }
}

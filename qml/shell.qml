import QtQuick
import Quickshell
import Quickshell.Io
import "theme"

ShellRoot {
    id: shellRoot

    FloatingWindow {
        id: win
        title: "QuickNews - Guvenli, Reklamsiz ve Resimsiz Haber Okuyucu"
        implicitWidth: 1400
        implicitHeight: 880
        color: Theme.bgBase

        MainWindow {
            id: mainWin
            anchors.fill: parent
        }
    }

    // Omarchy & CLI IPC Interface (Strict type safety and fast response)
    IpcHandler {
        target: "ozdil.quicknews"

        function toggle(): bool {
            win.visible = !win.visible;
            return win.visible;
        }

        function show(): bool {
            win.visible = true;
            return true;
        }

        function hide(): bool {
            win.visible = false;
            return false;
        }

        function sync(): string {
            mainWin.syncFeeds();
            return "OK";
        }

        function addPrompt(prompt: string): string {
            if (!prompt || prompt.trim().length === 0) return "Hata: bos arama istemi";
            mainWin.submitPromptAdd(prompt.trim());
            return "OK";
        }

        function reloadTheme(): string {
            Theme.lastLoadedRaw = "";
            Theme.colorsFile.reload();
            return "Tema yenilendi: " + Theme.themeName;
        }

        function getStatus(): string {
            var s = {
                "app": "QuickNews",
                "sourcesCount": mainWin.sourcesList.length,
                "articlesCount": mainWin.articlesList.length,
                "windowVisible": win.visible,
                "theme": Theme.themeName
            };
            return JSON.stringify(s);
        }
    }
}

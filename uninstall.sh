#!/bin/bash -p
set -euo pipefail

echo "=== QuickNews User-Space Removal Script ==="

STATE_DIR="${HOME}/.local/state/omarchy/quicknews"
MANIFEST_FILE="${STATE_DIR}/install_manifest.json"

if [[ ! -f "${MANIFEST_FILE}" ]]; then
    echo "Notice: No QuickNews installation manifest found at ${MANIFEST_FILE}."
    echo "Skipping ~/.local/bin deletion to preserve user-managed files."
else
    echo "Verifying QuickNews ownership from install manifest..."
    while IFS=$'\t' read -r file_path recorded_hash; do
        if [[ -z "${file_path}" || -z "${recorded_hash}" ]]; then
            continue
        fi

        if [[ -e "${file_path}" || -L "${file_path}" ]]; then
            if [[ -L "${file_path}" ]]; then
                echo "Warning: ${file_path} is a symlink. Skipping deletion to preserve user link." >&2
                continue
            fi
            current_hash=$(/usr/bin/sha256sum "${file_path}" 2>/dev/null | /usr/bin/awk '{print $1}' || echo "")
            if [[ "${current_hash}" == "${recorded_hash}" ]]; then
                echo "Verified QuickNews binary: ${file_path}. Removing..."
                /usr/bin/rm -f "${file_path}"
            else
                echo "Notice: ${file_path} does not match install hash (modified or replaced). Preserving file." >&2
            fi
        fi
    done < <(/usr/bin/python3 -c '
import json, sys
try:
    with open(sys.argv[1], "r", encoding="utf-8") as f:
        data = json.load(f)
        for path, digest in data.get("files", {}).items():
            print(f"{path}\t{digest}")
except Exception:
    pass
' "${MANIFEST_FILE}" 2>/dev/null || true)

    /usr/bin/rm -f "${MANIFEST_FILE}"
    echo "Installation manifest cleaned."
fi

# Clean installed desktop entry and shared quicknews QML assets if present
/usr/bin/rm -f "${HOME}/.local/share/applications/quicknews.desktop"
/usr/bin/rm -rf "${HOME}/.local/share/quicknews"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
fi

echo "QuickNews removal completed."

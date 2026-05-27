#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="${BIN_NAME:-nanobot}"
TARGET_TRIPLE="${TARGET_TRIPLE:?TARGET_TRIPLE is required}"
VERSION="${VERSION:?VERSION is required}"
TARGET_DIR="${TARGET_DIR:-target}"
DIST_DIR="${DIST_DIR:-dist}"

if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
  BIN_EXT=".exe"
  ARCHIVE_EXT=".zip"
else
  BIN_EXT=""
  ARCHIVE_EXT=".tar.gz"
fi

BIN_PATH="${TARGET_DIR}/${TARGET_TRIPLE}/release/${BIN_NAME}${BIN_EXT}"
STAGE_DIR="${DIST_DIR}/${BIN_NAME}-${VERSION}-${TARGET_TRIPLE}"
ARCHIVE_BASENAME="${BIN_NAME}-${VERSION}-${TARGET_TRIPLE}"
ARCHIVE_PATH="${DIST_DIR}/${ARCHIVE_BASENAME}${ARCHIVE_EXT}"

if [[ ! -f "${BIN_PATH}" ]]; then
  echo "release binary not found: ${BIN_PATH}" >&2
  exit 1
fi

rm -rf "${STAGE_DIR}"
mkdir -p "${STAGE_DIR}"
cp "${BIN_PATH}" "${STAGE_DIR}/${BIN_NAME}${BIN_EXT}"

if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
  powershell -NoProfile -Command \
    "Compress-Archive -Path '${STAGE_DIR}\\*' -DestinationPath '${ARCHIVE_PATH}' -Force" >/dev/null
else
  tar -czf "${ARCHIVE_PATH}" -C "${DIST_DIR}" "${ARCHIVE_BASENAME}"
fi

echo "archive=${ARCHIVE_PATH}" >> "${GITHUB_OUTPUT}"

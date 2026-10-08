#!/usr/bin/env bash
# Personal Apple Silicon build of Speech Flow.
# Ad-hoc signs the app (Tauri signingIdentity "-"). Does not notarize, and does
# not need an Apple Developer identity or an updater private key.
set -euo pipefail
cd "$(dirname "$0")/.."

unset APPLE_SIGNING_IDENTITY \
  APPLE_CERTIFICATE \
  APPLE_CERTIFICATE_PASSWORD \
  APPLE_ID \
  APPLE_PASSWORD \
  APPLE_ID_PASSWORD \
  APPLE_TEAM_ID \
  TAURI_SIGNING_PRIVATE_KEY \
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD

exec bun run tauri build --bundles app,dmg

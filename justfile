# Validate changelog history against the current project version.
api-generate:
    node scripts/generate-api.mjs

api-check:
    node scripts/generate-api.mjs --check

changelog-check:
    python3 scripts/changelog.py check

# Move Unreleased notes into a dated version; does not commit, tag or push.
release version:
    python3 scripts/changelog.py release '{{version}}'

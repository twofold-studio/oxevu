# Validate changelog history against the current project version.
changelog-check:
    python3 scripts/changelog.py check

# Move Unreleased notes into a dated version; does not commit, tag or push.
release version:
    python3 scripts/changelog.py release '{{version}}'

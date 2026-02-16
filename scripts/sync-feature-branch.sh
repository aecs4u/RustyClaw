#!/bin/bash
# Sync current feature branch with main
# Usage: ./scripts/sync-feature-branch.sh

set -e

CURRENT_BRANCH=$(git symbolic-ref --short HEAD 2>/dev/null)

if [ "$CURRENT_BRANCH" = "main" ]; then
    echo "❌ Already on main branch. Nothing to sync."
    exit 1
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔄 Syncing $CURRENT_BRANCH with main"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: You have uncommitted changes!"
    echo ""
    git status --short
    echo ""
    read -p "Stash changes and continue? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📦 Stashing changes..."
        git stash push -m "Auto-stash before sync with main"
        STASHED=true
    else
        echo "❌ Aborted. Please commit or stash your changes first."
        exit 1
    fi
fi

echo "🔄 Fetching latest main..."
git fetch origin main:main 2>/dev/null || echo "⚠️  Could not fetch origin/main"

echo "🔀 Merging main into $CURRENT_BRANCH..."
if git merge main --no-edit; then
    echo "✅ Branch synced successfully!"
    
    # Restore stashed changes if any
    if [ "$STASHED" = true ]; then
        echo "📦 Restoring stashed changes..."
        if git stash pop; then
            echo "✅ Changes restored"
        else
            echo "⚠️  Conflict while restoring stash. Resolve manually."
        fi
    fi
else
    echo ""
    echo "⚠️  MERGE CONFLICT DETECTED!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Conflicting files:"
    git diff --name-only --diff-filter=U | sed 's/^/  ❌ /'
    echo ""
    echo "To resolve:"
    echo "  1. Fix conflicts in the listed files"
    echo "  2. Stage resolved files: git add <file>"
    echo "  3. Complete merge: git commit"
    echo "  4. Or abort: git merge --abort"
    echo ""
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Sync complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

#!/bin/bash
# Create a new feature branch synced with main
# Usage: ./scripts/new-feature-branch.sh <branch-name>

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <branch-name>"
    echo "Example: $0 feature/hybrid-database"
    exit 1
fi

BRANCH_NAME="$1"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌱 Creating feature branch: $BRANCH_NAME"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Save current branch
CURRENT_BRANCH=$(git symbolic-ref --short HEAD 2>/dev/null || echo "detached")
echo "📍 Current branch: $CURRENT_BRANCH"

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo ""
    echo "⚠️  Warning: You have uncommitted changes!"
    echo ""
    git status --short
    echo ""
    read -p "Stash changes and continue? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "📦 Stashing changes..."
        git stash push -m "Auto-stash before creating $BRANCH_NAME"
        STASHED=true
    else
        echo "❌ Aborted. Please commit or stash your changes first."
        exit 1
    fi
fi

echo ""
echo "🔄 Step 1: Updating main branch..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Switch to main
git checkout main

# Pull latest changes from origin
echo "📥 Pulling latest changes from origin/main..."
if git pull origin main; then
    echo "✅ Main branch updated"
else
    echo "⚠️  Could not pull from origin (might be offline or no upstream)"
fi

echo ""
echo "🔄 Step 2: Creating feature branch..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create and checkout new branch from main
git checkout -b "$BRANCH_NAME"

echo "✅ Feature branch created: $BRANCH_NAME"
echo ""

# Show branch info
echo "📊 Branch Status:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Current branch: $(git symbolic-ref --short HEAD)"
echo "  Based on: main"
echo "  Latest commit: $(git log -1 --oneline main)"
echo ""

# Restore stashed changes if any
if [ "$STASHED" = true ]; then
    echo "📦 Restoring stashed changes..."
    if git stash pop; then
        echo "✅ Changes restored"
    else
        echo "⚠️  Conflict while restoring stash. Resolve manually with 'git stash pop'"
    fi
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 Ready to start development on $BRANCH_NAME!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 Tips:"
echo "  • Make commits regularly"
echo "  • Sync with main: git merge main"
echo "  • Push to remote: git push -u origin $BRANCH_NAME"
echo ""

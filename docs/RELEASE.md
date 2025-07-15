# Release Guide

This guide explains how to use the GitHub Actions release workflow and troubleshoot common issues.

## Release Workflow Overview

The release workflow supports two modes:

### 1. Automatic Releases (Push to Main)
- Triggered when code is merged to the `main` branch
- Creates a **draft prerelease** with version format: `v20250715-a1b2c3d`
- Must be manually published from GitHub UI

### 2. Manual Releases (GitHub UI)
- Triggered manually from GitHub Actions tab
- Allows custom version input (e.g., `v1.0.0`)
- Creates a **published release** marked as latest

## How to Create a Manual Release

1. Go to your repository on GitHub
2. Click the **Actions** tab
3. Click **Release** workflow in the left sidebar
4. Click **Run workflow** button (top right)
5. Enter version (e.g., `v1.0.0`)
6. Click **Run workflow**

## GitHub Token Permissions Issue

### Problem
The release workflow fails with a 403 error:
```
⚠️ GitHub release failed with status: 403
```

### Root Cause
The default `GITHUB_TOKEN` has insufficient permissions to create releases. GitHub requires explicit permissions for repository modifications.

### Solution Options

#### Option 1: Repository Settings (Recommended)
1. Go to **Settings** → **Actions** → **General**
2. Scroll to **Workflow permissions**
3. Select **Read and write permissions**
4. Check **Allow GitHub Actions to create and approve pull requests**
5. Click **Save**

#### Option 2: Workflow-Level Permissions
Add permissions to the release workflow:

```yaml
# In .github/workflows/release.yml
jobs:
  release:
    name: Create GitHub Release
    runs-on: ubuntu-latest
    needs: build
    permissions:
      contents: write  # Required for creating releases
    steps:
      # ... rest of the workflow
```

#### Option 3: Personal Access Token (If Above Fails)
1. Go to **Settings** → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
2. Click **Generate new token (classic)**
3. Select scopes: `repo` (full control)
4. Copy the token
5. Go to repository **Settings** → **Secrets and variables** → **Actions**
6. Click **New repository secret**
7. Name: `RELEASE_TOKEN`
8. Value: Your personal access token
9. Update the workflow to use `RELEASE_TOKEN` instead of `GITHUB_TOKEN`

## Release Artifacts

Each release includes:
- `rem-tui-macos.tar.gz` - Contains the executable and required dylib
- Release notes with installation instructions
- System requirements and usage information

## Installation Instructions (for users)

1. Download `rem-tui-macos.tar.gz` from the release page
2. Extract: `tar -xzf rem-tui-macos.tar.gz`
3. Move to PATH: `mv rem-tui /usr/local/bin/`
4. Ensure dylib is accessible (same directory or in library path)

## Requirements

- macOS 14.0 or later
- Reminders app permissions (requested on first run)

## Troubleshooting

### Release Creation Fails
- Check repository permissions (see above)
- Verify workflow has necessary permissions
- Ensure you're a repository maintainer

### Build Fails
- Check CI workflow status first
- Ensure all tests pass before releasing
- Review build logs for specific errors

### Missing Artifacts
- Verify build job completed successfully
- Check artifact upload/download steps
- Ensure distribution package creation succeeded

## Development Workflow

1. Make changes and create PR
2. CI runs automatically on PR
3. Merge PR to main
4. Automatic draft release created
5. Review and publish release when ready
6. Or create manual release with specific version

## Version Formats

- **Automatic**: `v20250715-a1b2c3d` (date + commit SHA)
- **Manual**: `v1.0.0`, `v1.2.3-beta`, etc. (semantic versioning)

## Best Practices

1. Use automatic releases for development builds
2. Use manual releases for stable versions
3. Test releases thoroughly before publishing
4. Include meaningful release notes
5. Follow semantic versioning for manual releases
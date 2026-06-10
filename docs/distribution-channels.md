# Distribution Channels

This document outlines the package distribution channels for Aether.

## Current Distribution

### GitHub Releases
- Primary distribution channel
- Automated on version tags
- Supports Windows, macOS, Linux, and Debian packages
- No account required for downloads

## Additional Distribution Channels

### Linux

#### Snap Store
**Prerequisites:**
- Snapcraft account
- Snapcraft token stored as GitHub Secret: `SNAPCRAFT_TOKEN`

**Setup:**
1. Create `snap/snapcraft.yaml`
2. Configure build and packaging
3. Add to release workflow:

```yaml
- name: Build Snap
  if: matrix.platform == 'ubuntu'
  run: |
    snapcraft

- name: Publish to Snap Store
  if: matrix.platform == 'ubuntu'
  run: |
    snapcraft upload --release=stable *.snap
  env:
    SNAPCRAFT_TOKEN: ${{ secrets.SNAPCRAFT_TOKEN }}
```

#### Flathub
**Prerequisites:**
- Flathub repository access
- GPG key for signing

**Setup:**
1. Create `flatpak/com.aether.app.json`
2. Submit to Flathub via pull request
3. Automated builds on Flathub infrastructure

#### Homebrew
**Prerequisites:**
- Homebrew repository access or tap
- GPG key for formula verification

**Setup:**
1. Create Homebrew formula in custom tap
2. Update formula on release:

```yaml
- name: Update Homebrew Formula
  run: |
    brew tap builds-toqyo/tap
    # Update formula with new version
    brew bump-formula-pr aether --version=${{ steps.version.outputs.VERSION }}
```

### Windows

#### Chocolatey
**Prerequisites:**
- Chocolatey maintainer account
- API key stored as GitHub Secret: `CHOCO_API_KEY`

**Setup:**
1. Create `aether.nuspec`
2. Create Chocolatey package
3. Publish to Chocolatey:

```yaml
- name: Publish to Chocolatey
  if: matrix.platform == 'windows'
  run: |
    choco push aether.nupkg --source https://push.chocolatey.org/ --api-key ${{ secrets.CHOCO_API_KEY }}
```

#### Microsoft Store
**Prerequisites:**
- Microsoft Developer account
- App submission process

**Setup:**
1. Package for Microsoft Store
2. Submit through Partner Center
3. Manual review process

### macOS

#### Homebrew Cask
**Prerequisites:**
- Homebrew repository access
- GPG key for cask verification

**Setup:**
1. Create cask in homebrew-cask tap
2. Update cask on release

```yaml
- name: Update Homebrew Cask
  run: |
    brew tap homebrew/cask
    brew bump-cask-version aether
```

#### Mac App Store
**Prerequisites:**
- Apple Developer Program membership
- App Store Connect access
- Mac App Store signing certificate

**Setup:**
1. Configure for Mac App Store distribution
2. Submit through App Store Connect
3. Manual review process

## Distribution Channel Priority

### Primary
- GitHub Releases (all platforms)

### Secondary (to implement)
- Snap Store (Linux)
- Flathub (Linux)
- Chocolatey (Windows)
- Homebrew (macOS)

### Tertiary (future consideration)
- Microsoft Store (Windows)
- Mac App Store (macOS)
- Steam (for distribution)

## Release Process

1. Tag release with version number
2. GitHub Actions builds all platforms
3. Packages uploaded to GitHub Releases
4. Distribution channels updated automatically
5. Documentation updated with release notes

## Verification

### Snap Verification
```bash
snap info aether
```

### Flatpak Verification
```bash
flatpak install flathub com.aether.app
```

### Chocolatey Verification
```bash
choco info aether
```

### Homebrew Verification
```bash
brew info aether
```

## Metrics and Analytics

Track downloads and installations from each channel:
- GitHub Releases download counts
- Snap Store install metrics
- Flathub download statistics
- Chocolatey package metrics
- Homebrew install counts

## Update Frequency

- Stable releases: Monthly or as needed
- Beta releases: Weekly during active development
- Security updates: Immediate when needed
- Distribution channel updates: Within 24 hours of release

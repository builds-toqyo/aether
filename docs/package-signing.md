# Package Signing Configuration

This document outlines the package signing setup for Aether.

## Windows Code Signing

### Prerequisites
- Code signing certificate from a trusted CA (DigiCert, Sectigo, etc.)
- Certificate stored as GitHub Secret: `WINDOWS_CERTIFICATE_BASE64`
- Certificate password stored as GitHub Secret: `WINDOWS_CERTIFICATE_PASSWORD`

### Signing Process
1. Extract certificate from base64-encoded secret
2. Install certificate in Windows certificate store
3. Sign the MSI installer using SignTool
4. Timestamp the signature

### GitHub Actions Configuration
Add to release.yml after build step:

```yaml
- name: Setup Windows Code Signing
  if: matrix.platform == 'windows'
  run: |
    echo ${{ secrets.WINDOWS_CERTIFICATE_BASE64 }} | base64 -d > cert.pfx
    certutil -importPFX cert.pfx ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}

- name: Sign Windows Installer
  if: matrix.platform == 'windows'
  run: |
    signtool sign /f cert.pfx /p ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }} /tr http://timestamp.digicert.com /td sha256 /fd sha256 src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/*.msi
```

## macOS Code Signing & Notarization

### Prerequisites
- Apple Developer account
- Developer certificate and provisioning profile
- App-specific password stored as GitHub Secret: `APPLE_APP_SPECIFIC_PASSWORD`
- Apple ID stored as GitHub Secret: `APPLE_ID`
- Team ID stored as GitHub Secret: `APPLE_TEAM_ID`

### Signing Process
1. Import developer certificate
2. Sign the DMG bundle
3. Submit for notarization to Apple
4. Staple notarization ticket to DMG

### GitHub Actions Configuration
Add to release.yml after build step:

```yaml
- name: Import macOS Signing Certificate
  if: matrix.platform == 'macos'
  run: |
    echo ${{ secrets.MACOS_CERTIFICATE_BASE64 }} | base64 -d > certificate.p12
    security create-keychain -p "" build.keychain
    security import certificate.p12 -k ~/Library/Keychains/build.keychain -P ${{ secrets.MACOS_CERTIFICATE_PASSWORD }}
    security set-key-partition-list -S apple-tool:,apple: -s -k "" ~/Library/Keychains/build.keychain
    security default-keychain -s ~/Library/Keychains/build.keychain
    security unlock-keychain -p "" ~/Library/Keychains/build.keychain
    security set-keychain-settings -l ~/Library/Keychains/build.keychain

- name: Sign macOS DMG
  if: matrix.platform == 'macos'
  run: |
    codesign --force --deep --sign ${{ secrets.MACOS_SIGNING_IDENTITY }} src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/*.dmg

- name: Notarize macOS DMG
  if: matrix.platform == 'macos'
  run: |
    xcrun notarytool submit src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/*.dmg \
      --apple-id ${{ secrets.APPLE_ID }} \
      --password ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }} \
      --team-id ${{ secrets.APPLE_TEAM_ID }} \
      --wait
    xcrun stapler staple src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/*.dmg
```

## Linux Package Signing

### Prerequisites
- GPG key for package signing
- Public key uploaded to keyserver (keyserver.ubuntu.com)
- Private key stored as GitHub Secret: `LINUX_GPG_PRIVATE_KEY`
- Passphrase stored as GitHub Secret: `LINUX_GPG_PASSPHRASE`

### Signing Process
1. Import GPG private key
2. Sign DEB packages using dpkg-sig
3. Sign AppImage using GPG
4. Upload public key to keyserver

### GitHub Actions Configuration
Add to release.yml after build step:

```yaml
- name: Import GPG Key
  if: matrix.platform == 'ubuntu' || matrix.platform == 'debian'
  run: |
    echo ${{ secrets.LINUX_GPG_PRIVATE_KEY }} | base64 -d > private.key
    gpg --import private.key

- name: Sign DEB Package
  if: (matrix.platform == 'ubuntu' || matrix.platform == 'debian') && matrix.package_format == 'deb'
  run: |
    dpkg-sig --sign builder -k ${{ secrets.LINUX_GPG_KEY_ID }} src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/deb/*.deb

- name: Sign AppImage
  if: matrix.platform == 'ubuntu' && matrix.package_format == 'appimage'
  run: |
    gpg --detach-sign --armor --local-user ${{ secrets.LINUX_GPG_KEY_ID }} /tmp/*.AppImage
```

## Docker Image Signing

### Prerequisites
- Docker Hub account
- GitHub Container Registry access
- Cosign for image signing

### Signing Process
1. Build Docker image
2. Sign using Cosign
3. Push to registry

### GitHub Actions Configuration
Add to release.yml after docker build:

```yaml
- name: Install Cosign
  run: |
    curl -L https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64 -o cosign
    chmod +x cosign
    sudo mv cosign /usr/local/bin/

- name: Sign Docker Image
  run: |
    cosign sign -y ghcr.io/${{ github.repository }}:${{ github.ref_name }}
```

## Verification

### Verify Windows Signature
```bash
signtool verify /pa /v Aether_Setup.exe
```

### Verify macOS Signature
```bash
codesign -dv --verbose=4 Aether.dmg
spctl -a -t exec -vv Aether.dmg
```

### Verify Linux Signature
```bash
gpg --verify aether.deb.asc aether.deb
dpkg-sig --verify aether.deb
```

### Verify Docker Signature
```bash
cosign verify ghcr.io/builds-toqyo/aether:latest
```

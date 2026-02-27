# ProxyPal — Hướng dẫn Đóng gói & Phân phối macOS

> **Dự án:** ProxyPal (Tauri v2 + SolidJS + Rust)
> **Identifier:** `com.proxypal.app`
> **Phiên bản hiện tại:** 0.4.5
> **macOS Minimum:** 10.15 (Catalina)

---

## Mục lục

1. [Thiết lập Môi trường Build](#1-thiết-lập-môi-trường-build)
2. [Tối ưu hóa Release Build](#2-tối-ưu-hóa-release-build)
3. [Biên dịch Mã nguồn Production](#3-biên-dịch-mã-nguồn-production)
4. [Universal Binary — Đa kiến trúc](#4-universal-binary--đa-kiến-trúc)
5. [Apple Code Signing & Notarization](#5-apple-code-signing--notarization)
6. [Tạo DMG/PKG](#6-tạo-dmgpkg)
7. [Tự động hóa với GitHub Actions](#7-tự-động-hóa-với-github-actions)

---

## 1. Thiết lập Môi trường Build

### 1.1. Yêu cầu hệ thống

| Công cụ | Phiên bản tối thiểu | Ghi chú |
|---------|---------------------|---------|
| macOS | 11.0+ (Big Sur) | Máy build; ứng dụng hỗ trợ từ 10.15 |
| Xcode CLT | 14+ | `xcode-select --install` |
| Rust | 1.77+ (stable) | Cần cả target `aarch64` và `x86_64` |
| Node.js | 20 LTS | Khuyến nghị dùng `nvm` hoặc `fnm` |
| pnpm | 9.x | Package manager chính của dự án |
| Tauri CLI | 2.10+ | Đã cài qua devDependencies |

### 1.2. Cài đặt từng bước

```bash
# 1. Xcode Command Line Tools
xcode-select --install

# 2. Rust (qua rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Thêm cả hai target cho macOS
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin

# 3. Node.js 20 LTS (qua fnm — khuyến nghị)
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 20
fnm use 20

# 4. pnpm
corepack enable
corepack prepare pnpm@9 --activate

# 5. Cài dependencies của dự án
pnpm install
```

### 1.3. Xác minh môi trường

```bash
# Kiểm tra tất cả công cụ
rustc --version          # rustc 1.XX.0
cargo --version          # cargo 1.XX.0
node --version           # v20.XX.X
pnpm --version           # 9.X.X
pnpm tauri --version     # tauri-cli 2.10.X

# Kiểm tra Rust targets đã cài
rustup target list --installed
# Phải thấy:
#   aarch64-apple-darwin
#   x86_64-apple-darwin
```

### 1.4. Sidecar Binary

ProxyPal sử dụng một sidecar binary (`cli-proxy-api`) được cấu hình trong `tauri.conf.json`:

```json
"externalBin": ["binaries/cli-proxy-api"]
```

Binary này cần được đặt đúng tên theo target triple trong thư mục `src-tauri/binaries/`:

| Target | Tên file |
|--------|----------|
| `aarch64-apple-darwin` | `cli-proxy-api-aarch64-apple-darwin` |
| `x86_64-apple-darwin` | `cli-proxy-api-x86_64-apple-darwin` |

> **Lưu ý:** `build.rs` của dự án tự động xử lý việc tải sidecar binary. Xem script `scripts/update-sidecar.mjs` để tải thủ công.

---

## 2. Tối ưu hóa Release Build

### 2.1. Rust — Cargo.toml

Thêm profile `[profile.release]` vào `src-tauri/Cargo.toml` để tối ưu kích thước và hiệu suất:

```toml
[profile.release]
# Tối ưu kích thước binary (giảm 30-50%)
strip = true           # Xóa debug symbols
lto = true             # Link-Time Optimization — chậm hơn nhưng nhỏ hơn đáng kể
opt-level = "s"        # Tối ưu cho kích thước ("s") hoặc tốc độ ("3")
codegen-units = 1      # Biên dịch đơn luồng — tối ưu hơn nhưng chậm hơn
panic = "abort"        # Không cần unwind — giảm kích thước binary
```

**Giải thích các tùy chọn:**

| Tùy chọn | Giá trị khuyến nghị | Tác dụng |
|-----------|---------------------|----------|
| `strip` | `true` | Xóa debug symbols, giảm ~30% kích thước |
| `lto` | `true` (fat LTO) | Tối ưu xuyên crate, giảm thêm ~20% |
| `opt-level` | `"s"` hoặc `"z"` | `"s"` = tối ưu kích thước; `"z"` = cực nhỏ; `"3"` = cực nhanh |
| `codegen-units` | `1` | Cho phép LTO tối ưu toàn bộ, build chậm hơn |
| `panic` | `"abort"` | Bỏ unwind machinery; Tauri không cần catch_unwind |

> **Trade-off:** `lto = true` + `codegen-units = 1` tăng thời gian build release lên 2-3x nhưng cho binary nhỏ hơn đáng kể.

### 2.2. Frontend — Vite Config

File `vite.config.ts` hiện tại đã khá tốt. Để tối ưu thêm cho production, có thể bổ sung:

```typescript
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  base: "./",
  plugins: [solid()],

  // === Tối ưu Production Build ===
  build: {
    // Tối ưu kích thước chunk
    target: "esnext",         // Tauri dùng WebView hiện đại — không cần polyfill
    minify: "terser",         // terser nhỏ hơn esbuild ~5-10%
    terserOptions: {
      compress: {
        drop_console: true,   // Xóa console.log trong production
        drop_debugger: true,
      },
    },
    // Tách chunk hợp lý
    rollupOptions: {
      output: {
        manualChunks: {
          echarts: ["echarts"],
          chartjs: ["chart.js"],
          kobalte: ["@kobalte/core"],
        },
      },
    },
    // Giới hạn cảnh báo chunk size
    chunkSizeWarningLimit: 600,
  },

  // Phần còn lại giữ nguyên...
  clearScreen: false,
  server: {
    hmr: host ? { host, port: 1421, protocol: "ws" } : undefined,
    host: host || false,
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
```

### 2.3. Tauri Bundle Config

Trong `tauri.conf.json`, kiểm tra và bổ sung:

```jsonc
{
  "bundle": {
    "active": true,
    "targets": ["app", "dmg"],   // Chỉ target macOS khi build trên Mac
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "DeveloperTool",
    "macOS": {
      "minimumSystemVersion": "10.15",
      // Frameworks cần bundle (nếu có)
      "frameworks": [],
      // Entitlements file (cần cho notarization)
      "entitlements": null,
      // Signing identity (xem phần 5)
      "signingIdentity": null
    }
  }
}
```

---

## 3. Biên dịch Mã nguồn Production

### 3.1. Build cơ bản

```bash
# Build cho kiến trúc hiện tại (native)
pnpm tauri build
```

Lệnh này tự động:
1. Chạy `pnpm build` (Vite production build → `dist/`)
2. Biên dịch Rust với `cargo build --release`
3. Tạo bundle `.app` + `.dmg` trong `src-tauri/target/release/bundle/`

### 3.2. Build cho target cụ thể

```bash
# Apple Silicon (M1/M2/M3/M4)
pnpm tauri build --target aarch64-apple-darwin

# Intel
pnpm tauri build --target x86_64-apple-darwin
```

### 3.3. Cấu trúc output

```
src-tauri/target/<target-triple>/release/
├── ProxyPal                          # Binary thực thi
├── bundle/
│   ├── macos/
│   │   └── ProxyPal.app/            # macOS Application Bundle
│   │       └── Contents/
│   │           ├── Info.plist
│   │           ├── MacOS/
│   │           │   ├── ProxyPal     # Main binary
│   │           │   └── cli-proxy-api # Sidecar
│   │           ├── Resources/
│   │           │   └── icon.icns
│   │           └── _CodeSignature/
│   └── dmg/
│       └── ProxyPal_0.4.5_<arch>.dmg
```

### 3.4. Kiểm tra build output

```bash
TARGET="aarch64-apple-darwin"  # hoặc x86_64-apple-darwin
BUILD_DIR="src-tauri/target/${TARGET}/release"

# Kiểm tra binary
file "${BUILD_DIR}/ProxyPal"

# Kiểm tra .app bundle
ls -la "${BUILD_DIR}/bundle/macos/ProxyPal.app/Contents/MacOS/"

# Kiểm tra kích thước
du -sh "${BUILD_DIR}/bundle/macos/ProxyPal.app"
du -sh "${BUILD_DIR}/bundle/dmg/"*.dmg

# Kiểm tra kiến trúc
lipo -archs "${BUILD_DIR}/bundle/macos/ProxyPal.app/Contents/MacOS/ProxyPal"
```

---

## 4. Universal Binary — Đa kiến trúc

### 4.1. Tổng quan

Universal Binary cho phép một file `.app` chạy native trên cả Intel và Apple Silicon. Tauri v2 **không tự động** tạo Universal Binary — cần build riêng từng target rồi merge thủ công bằng `lipo`.

### 4.2. Phương pháp 1: Merge thủ công bằng lipo

```bash
#!/bin/bash
set -euo pipefail

VERSION="0.4.5"
PRODUCT="ProxyPal"

# --- Bước 1: Build cho từng kiến trúc ---
echo "=== Building for Apple Silicon ==="
pnpm tauri build --target aarch64-apple-darwin

echo "=== Building for Intel ==="
pnpm tauri build --target x86_64-apple-darwin

# --- Bước 2: Đường dẫn output ---
ARM_APP="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/${PRODUCT}.app"
X86_APP="src-tauri/target/x86_64-apple-darwin/release/bundle/macos/${PRODUCT}.app"
UNIVERSAL_APP="src-tauri/target/universal-apple-darwin/release/bundle/macos/${PRODUCT}.app"

# --- Bước 3: Tạo thư mục Universal ---
mkdir -p "$(dirname "$UNIVERSAL_APP")"
cp -R "$ARM_APP" "$UNIVERSAL_APP"

# --- Bước 4: Merge main binary ---
lipo -create \
  "${ARM_APP}/Contents/MacOS/${PRODUCT}" \
  "${X86_APP}/Contents/MacOS/${PRODUCT}" \
  -output "${UNIVERSAL_APP}/Contents/MacOS/${PRODUCT}"

# --- Bước 5: Merge sidecar (cli-proxy-api) ---
# Nếu sidecar cũng cần Universal:
if [ -f "${ARM_APP}/Contents/MacOS/cli-proxy-api" ] && \
   [ -f "${X86_APP}/Contents/MacOS/cli-proxy-api" ]; then
  lipo -create \
    "${ARM_APP}/Contents/MacOS/cli-proxy-api" \
    "${X86_APP}/Contents/MacOS/cli-proxy-api" \
    -output "${UNIVERSAL_APP}/Contents/MacOS/cli-proxy-api"
  echo "Sidecar merged as Universal Binary"
fi

# --- Bước 6: Xác minh ---
echo "=== Verification ==="
lipo -archs "${UNIVERSAL_APP}/Contents/MacOS/${PRODUCT}"
# Expected output: x86_64 arm64

file "${UNIVERSAL_APP}/Contents/MacOS/${PRODUCT}"
# Expected: Mach-O universal binary with 2 architectures:
#   x86_64, arm64

echo "Universal .app created at: ${UNIVERSAL_APP}"
```

### 4.3. Phương pháp 2: Build riêng biệt — Khuyến nghị

Dự án ProxyPal hiện tại sử dụng phương pháp này trong CI/CD (xem `.github/workflows/release.yml`):

- Build riêng cho `aarch64-apple-darwin` và `x86_64-apple-darwin`
- Phân phối 2 file DMG riêng biệt
- Người dùng tải đúng bản cho kiến trúc của họ

**Ưu điểm:**
- File DMG nhỏ hơn (chỉ chứa 1 kiến trúc)
- Sidecar binary không cần merge (tránh vấn đề tương thích)
- Đơn giản hơn cho CI/CD

**Nhược điểm:**
- Người dùng phải chọn đúng phiên bản
- Cần host 2 file riêng biệt

### 4.4. Lưu ý với Sidecar

ProxyPal bundle sidecar `cli-proxy-api` — đây là binary bên ngoài được tải từ repo `CLIProxyAPIPlus`. Khi tạo Universal Binary:

1. **Sidecar phải tương thích kiến trúc** với main binary
2. Nếu sidecar chỉ có bản riêng (arm64/x86_64), dùng phương pháp 2
3. Nếu muốn Universal, cần `lipo -create` cả sidecar

---

## 5. Apple Code Signing & Notarization

### 5.1. Tổng quan quy trình

```
┌─────────────────┐     ┌──────────────┐     ┌──────────────┐     ┌─────────────┐
│  Build .app     │────▶│  Code Sign   │────▶│  Notarize    │────▶│  Staple     │
│  (Tauri build)  │     │  (codesign)  │     │  (notarytool)│     │  (stapler)  │
└─────────────────┘     └──────────────┘     └──────────────┘     └─────────────┘
```

### 5.2. Chuẩn bị trên Apple Developer Portal

#### a. Đăng ký Apple Developer Program

- Truy cập [developer.apple.com](https://developer.apple.com)
- Đăng ký tài khoản ($99/năm cho cá nhân)
- Cần cho Code Signing & Notarization

#### b. Tạo Certificate

1. Mở **Keychain Access** → **Certificate Assistant** → **Request a Certificate from a Certificate Authority**
2. Điền email, chọn **Saved to disk**
3. Truy cập [Apple Developer Portal > Certificates](https://developer.apple.com/account/resources/certificates/list)
4. Tạo certificate loại **Developer ID Application** (cho phân phối ngoài App Store)
5. Upload CSR file, download `.cer`, double-click để import vào Keychain

#### c. Tạo App ID

1. Truy cập [Identifiers](https://developer.apple.com/account/resources/identifiers/list)
2. Tạo **App ID** mới:
   - **Bundle ID:** `com.proxypal.app` (khớp với `identifier` trong `tauri.conf.json`)
   - **Platform:** macOS
3. Chọn Capabilities cần thiết (nếu có)

#### d. Tạo App-Specific Password

Dùng cho notarization tự động (thay vì mật khẩu Apple ID):

1. Truy cập [appleid.apple.com](https://appleid.apple.com)
2. **Sign-In and Security** → **App-Specific Passwords**
3. Tạo password mới, lưu lại (ví dụ: `xxxx-xxxx-xxxx-xxxx`)

### 5.3. Cấu hình Code Signing trong Tauri

#### a. Biến môi trường

Tauri v2 sử dụng biến môi trường để cấu hình signing:

```bash
# Apple Developer Team ID (10 ký tự)
export APPLE_TEAM_ID="XXXXXXXXXX"

# Signing identity (tên certificate trong Keychain)
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (XXXXXXXXXX)"

# Apple ID cho notarization
export APPLE_ID="your-email@example.com"

# App-Specific Password
export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"

# Hoặc lưu password vào Keychain (khuyến nghị)
xcrun notarytool store-credentials "ProxyPal-Notarize" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_PASSWORD"
```

#### b. tauri.conf.json

```jsonc
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "10.15",
      // Signing identity — nếu null, Tauri tự detect từ Keychain
      "signingIdentity": null,
      // Entitlements cho hardened runtime (bắt buộc cho notarization)
      "entitlements": "./Entitlements.plist"
    }
  }
}
```

#### c. Tạo Entitlements.plist

Tạo file `src-tauri/Entitlements.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Cho phép JIT (cần cho WebView) -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>

    <!-- Cho phép unsigned executable memory (WebView cần) -->
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>

    <!-- Cho phép DYLD env variables (debug, tùy chọn) -->
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>

    <!-- Network access (ProxyPal cần) -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>

    <!-- File access (cho config, logs) -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
</dict>
</plist>
```

### 5.4. Quy trình Notarization

#### a. Tự động qua Tauri CLI

Tauri v2 hỗ trợ notarization tự động khi có đủ biến môi trường:

```bash
# Đặt biến môi trường
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (XXXXXXXXXX)"
export APPLE_ID="your-email@example.com"
export APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"
export APPLE_TEAM_ID="XXXXXXXXXX"

# Build + Sign + Notarize trong 1 lệnh
pnpm tauri build --target aarch64-apple-darwin
```

Tauri sẽ tự động:
1. Code sign `.app` bundle với hardened runtime
2. Tạo `.dmg`
3. Gửi `.dmg` đến Apple Notary Service
4. Đợi kết quả (thường 2-15 phút)
5. Staple notarization ticket vào `.dmg`

#### b. Thủ công bằng Terminal

Nếu cần notarize thủ công (ví dụ: build đã có sẵn):

```bash
# === Bước 1: Code Sign .app ===
codesign --force --deep --options runtime \
  --sign "Developer ID Application: Your Name (XXXXXXXXXX)" \
  --entitlements src-tauri/Entitlements.plist \
  "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ProxyPal.app"

# Xác minh signature
codesign --verify --deep --strict --verbose=2 \
  "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ProxyPal.app"

# === Bước 2: Code Sign sidecar riêng (nếu cần) ===
codesign --force --options runtime \
  --sign "Developer ID Application: Your Name (XXXXXXXXXX)" \
  "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ProxyPal.app/Contents/MacOS/cli-proxy-api"

# === Bước 3: Tạo DMG ===
hdiutil create -volname "ProxyPal" -srcfolder \
  "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ProxyPal.app" \
  -ov -format UDZO \
  "ProxyPal_0.4.5_aarch64.dmg"

# Code Sign DMG
codesign --force --sign "Developer ID Application: Your Name (XXXXXXXXXX)" \
  "ProxyPal_0.4.5_aarch64.dmg"

# === Bước 4: Notarize ===
# Sử dụng stored credentials
xcrun notarytool submit "ProxyPal_0.4.5_aarch64.dmg" \
  --keychain-profile "ProxyPal-Notarize" \
  --wait

# Hoặc sử dụng trực tiếp credentials
xcrun notarytool submit "ProxyPal_0.4.5_aarch64.dmg" \
  --apple-id "your-email@example.com" \
  --team-id "XXXXXXXXXX" \
  --password "xxxx-xxxx-xxxx-xxxx" \
  --wait

# === Bước 5: Staple ===
xcrun stapler staple "ProxyPal_0.4.5_aarch64.dmg"

# === Bước 6: Xác minh ===
spctl --assess --type open --context context:primary-signature \
  "ProxyPal_0.4.5_aarch64.dmg"

xcrun stapler validate "ProxyPal_0.4.5_aarch64.dmg"
```

### 5.5. Xử lý lỗi thường gặp

| Lỗi | Nguyên nhân | Giải pháp |
|-----|-------------|-----------|
| `errSecInternalComponent` | Keychain bị khóa | `security unlock-keychain -p PASSWORD login.keychain` |
| `The signature of the binary is invalid` | Sidecar không được sign | Sign sidecar trước khi sign .app |
| `Notarization failed: hardened runtime` | Thiếu `--options runtime` | Thêm flag `--options runtime` khi codesign |
| `The executable does not have the hardened runtime enabled` | Thiếu entitlements | Tạo và trỏ đúng file `Entitlements.plist` |
| `Package Invalid` (Notarization) | Thiếu entitlement cho JIT | Thêm `com.apple.security.cs.allow-jit` |

### 5.6. Lưu trữ Credentials an toàn trong CI

```bash
# Tạo temporary keychain cho CI
security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security default-keychain -s build.keychain
security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain

# Import certificate (.p12)
security import certificate.p12 \
  -k build.keychain \
  -P "$CERTIFICATE_PASSWORD" \
  -T /usr/bin/codesign \
  -T /usr/bin/security

# Cho phép codesign truy cập không cần prompt
security set-key-partition-list -S apple-tool:,apple: \
  -s -k "$KEYCHAIN_PASSWORD" build.keychain

# Lưu notarization credentials
xcrun notarytool store-credentials "ProxyPal-Notarize" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_PASSWORD" \
  --keychain build.keychain
```

---

## 6. Tạo DMG/PKG

### 6.1. DMG — Cấu hình Tauri

`tauri.conf.json` hiện tại đã cấu hình để tạo DMG:

```jsonc
{
  "bundle": {
    "targets": ["app", "dmg"],    // "dmg" đã có trong config hiện tại
    "macOS": {
      "minimumSystemVersion": "10.15"
    }
  }
}
```

Tauri v2 tự động tạo DMG khi build trên macOS. Output:

```
src-tauri/target/<target>/release/bundle/dmg/
  ProxyPal_0.4.5_<arch>.dmg
```

### 6.2. DMG tùy chỉnh (nâng cao)

Nếu muốn DMG đẹp hơn với background image, icon layout, v.v., dùng `create-dmg`:

```bash
# Cài đặt
brew install create-dmg

# Tạo DMG tùy chỉnh
create-dmg \
  --volname "ProxyPal" \
  --volicon "src-tauri/icons/icon.icns" \
  --background "docs/dmg-background.png" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "ProxyPal.app" 150 200 \
  --app-drop-link 450 200 \
  --hide-extension "ProxyPal.app" \
  "ProxyPal_0.4.5_aarch64.dmg" \
  "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ProxyPal.app"
```

### 6.3. PKG (Installer Package)

Tauri v2 không tạo PKG mặc định, nhưng có thể tạo thủ công:

```bash
# Tạo PKG từ .app bundle
productbuild \
  --component "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ProxyPal.app" \
  /Applications \
  --sign "Developer ID Installer: Your Name (XXXXXXXXXX)" \
  "ProxyPal_0.4.5_aarch64.pkg"

# Notarize PKG
xcrun notarytool submit "ProxyPal_0.4.5_aarch64.pkg" \
  --keychain-profile "ProxyPal-Notarize" \
  --wait

xcrun stapler staple "ProxyPal_0.4.5_aarch64.pkg"
```

> **Lưu ý:** PKG cần certificate loại **Developer ID Installer** (khác với Application). Tạo tại Apple Developer Portal > Certificates.

### 6.4. So sánh DMG vs PKG

| Tiêu chí | DMG | PKG |
|----------|-----|-----|
| Trải nghiệm | Kéo thả vào Applications | Installer wizard |
| Kích thước | Nhỏ hơn (nén tốt) | Lớn hơn một chút |
| Pre/Post-install scripts | Không | Có |
| MDM/Enterprise deploy | Hạn chế | Tốt hơn |
| Khuyến nghị cho ProxyPal | ✅ Phù hợp | Cần khi deploy MDM |

---

## 7. Tự động hóa với GitHub Actions

### 7.1. Workflow hiện tại

Dự án đã có `.github/workflows/release.yml` xử lý:

- Build multi-platform: `aarch64-apple-darwin`, `x86_64-apple-darwin`, Linux, Windows
- Tải sidecar binary tự động từ `CLIProxyAPIPlus`
- Dùng `tauri-apps/tauri-action@v0` để build và publish release
- Updater signing với `TAURI_SIGNING_PRIVATE_KEY`

### 7.2. Workflow bổ sung — Code Signing & Notarization

Dưới đây là ví dụ workflow mở rộng thêm Apple Code Signing và Notarization:

```yaml
name: Release (with Signing & Notarization)

on:
  push:
    tags:
      - "v*"

env:
  CARGO_TERM_COLOR: always

jobs:
  release-macos:
    name: macOS Release (${{ matrix.target }})
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-15-intel
            target: x86_64-apple-darwin
    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "pnpm"

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Rust cache
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install dependencies
        run: pnpm install

      # --- Sidecar Download (giữ nguyên logic hiện tại) ---
      - name: Download CLIProxyAPIPlus binary
        shell: bash
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          set -e
          mkdir -p src-tauri/binaries
          # ... (giữ nguyên logic download từ release.yml hiện tại)

      # --- Apple Code Signing Setup ---
      - name: Setup Apple Certificate
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          KEYCHAIN_PASSWORD: ${{ secrets.KEYCHAIN_PASSWORD }}
        run: |
          # Decode certificate
          echo "$APPLE_CERTIFICATE" | base64 --decode > certificate.p12

          # Create temporary keychain
          security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain

          # Import certificate
          security import certificate.p12 \
            -k build.keychain \
            -P "$APPLE_CERTIFICATE_PASSWORD" \
            -T /usr/bin/codesign \
            -T /usr/bin/security

          # Allow codesign to access keychain
          security set-key-partition-list -S apple-tool:,apple: \
            -s -k "$KEYCHAIN_PASSWORD" build.keychain

          # Cleanup
          rm certificate.p12

      # --- Build with Signing & Notarization ---
      - name: Build, Sign & Notarize
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          # Tauri Updater Signing
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          # Apple Code Signing
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          # Apple Notarization
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "ProxyPal ${{ github.ref_name }}"
          releaseBody: "See commits for details."
          releaseDraft: false
          prerelease: false
          args: --target ${{ matrix.target }}

      # --- Cleanup ---
      - name: Cleanup keychain
        if: always()
        run: |
          security delete-keychain build.keychain 2>/dev/null || true
```

### 7.3. GitHub Secrets cần cấu hình

Truy cập **Repository > Settings > Secrets and variables > Actions** và thêm:

| Secret | Giá trị | Cách tạo |
|--------|---------|----------|
| `APPLE_CERTIFICATE` | Base64 của `.p12` | `base64 -i Certificates.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Mật khẩu export `.p12` | Tự đặt khi export từ Keychain |
| `APPLE_SIGNING_IDENTITY` | Ví dụ: `Developer ID Application: Name (TEAMID)` | Xem trong Keychain Access |
| `APPLE_ID` | Email Apple Developer | Email đăng ký |
| `APPLE_PASSWORD` | App-Specific Password | Tạo tại appleid.apple.com |
| `APPLE_TEAM_ID` | 10 ký tự Team ID | Apple Developer Portal > Membership |
| `KEYCHAIN_PASSWORD` | Password tạm cho CI keychain | Tự tạo chuỗi ngẫu nhiên |
| `TAURI_SIGNING_PRIVATE_KEY` | Private key cho updater | Đã cấu hình |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password cho key trên | Đã cấu hình |

### 7.4. Bash Script đơn giản (Build local)

Dùng cho build thủ công trên máy local:

```bash
#!/bin/bash
# scripts/build-macos.sh — Build ProxyPal cho macOS với signing
set -euo pipefail

# === Cấu hình ===
TARGET="${1:-aarch64-apple-darwin}"   # Arg 1: target (mặc định Apple Silicon)
SIGN="${SIGN:-false}"                  # Đặt SIGN=true để code sign
VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*: "\(.*\)".*/\1/')

echo "================================================"
echo " ProxyPal macOS Build"
echo " Version: ${VERSION}"
echo " Target:  ${TARGET}"
echo " Sign:    ${SIGN}"
echo "================================================"

# === Bước 1: Kiểm tra prerequisites ===
command -v rustc >/dev/null || { echo "ERROR: Rust not installed"; exit 1; }
command -v pnpm >/dev/null || { echo "ERROR: pnpm not installed"; exit 1; }

# Kiểm tra target đã cài
if ! rustup target list --installed | grep -q "$TARGET"; then
  echo "Installing Rust target: $TARGET"
  rustup target add "$TARGET"
fi

# === Bước 2: Kiểm tra sidecar ===
SIDECAR="src-tauri/binaries/cli-proxy-api-${TARGET}"
if [ ! -f "$SIDECAR" ]; then
  echo "WARNING: Sidecar binary not found at ${SIDECAR}"
  echo "Run: pnpm update-sidecar"
  read -p "Continue without sidecar? [y/N] " -n 1 -r
  echo
  [[ $REPLY =~ ^[Yy]$ ]] || exit 1
fi

# === Bước 3: Build ===
echo ""
echo ">>> Building frontend (Vite)..."
pnpm build

echo ""
echo ">>> Building Tauri app for ${TARGET}..."
if [ "$SIGN" = "true" ]; then
  # Kiểm tra signing env vars
  : "${APPLE_SIGNING_IDENTITY:?Set APPLE_SIGNING_IDENTITY}"
  : "${APPLE_ID:?Set APPLE_ID for notarization}"
  : "${APPLE_PASSWORD:?Set APPLE_PASSWORD for notarization}"
  : "${APPLE_TEAM_ID:?Set APPLE_TEAM_ID for notarization}"

  pnpm tauri build --target "$TARGET"
  echo "Build completed with signing & notarization"
else
  pnpm tauri build --target "$TARGET"
  echo "Build completed (unsigned)"
fi

# === Bước 4: Output ===
BUILD_DIR="src-tauri/target/${TARGET}/release/bundle"
echo ""
echo "================================================"
echo " Build Output:"
echo "================================================"

if [ -d "${BUILD_DIR}/macos" ]; then
  APP_SIZE=$(du -sh "${BUILD_DIR}/macos/ProxyPal.app" 2>/dev/null | cut -f1)
  echo " .app: ${BUILD_DIR}/macos/ProxyPal.app (${APP_SIZE})"
fi

if [ -d "${BUILD_DIR}/dmg" ]; then
  DMG_FILE=$(ls "${BUILD_DIR}/dmg/"*.dmg 2>/dev/null | head -1)
  if [ -n "$DMG_FILE" ]; then
    DMG_SIZE=$(du -sh "$DMG_FILE" | cut -f1)
    echo " .dmg: ${DMG_FILE} (${DMG_SIZE})"
  fi
fi

# Kiểm tra architecture
echo ""
echo " Architecture check:"
lipo -archs "${BUILD_DIR}/macos/ProxyPal.app/Contents/MacOS/ProxyPal" 2>/dev/null || echo "  (could not verify)"

echo ""
echo "Done!"
```

Cách sử dụng:

```bash
# Build cho Apple Silicon (mặc định, không sign)
bash scripts/build-macos.sh

# Build cho Intel
bash scripts/build-macos.sh x86_64-apple-darwin

# Build với Code Signing & Notarization
SIGN=true bash scripts/build-macos.sh aarch64-apple-darwin
```

---

## Phụ lục

### A. Checklist trước khi Release

- [ ] Version đã cập nhật trong `package.json`, `Cargo.toml`, `tauri.conf.json`
- [ ] `pnpm tsc --noEmit` pass
- [ ] `cd src-tauri && cargo check` pass
- [ ] `pnpm test` pass
- [ ] Sidecar binary đúng version và kiến trúc
- [ ] (Nếu sign) Certificate chưa hết hạn
- [ ] (Nếu sign) Entitlements.plist đúng permissions
- [ ] Build thành công trên cả arm64 và x86_64
- [ ] DMG mở được và cài đặt thành công
- [ ] App chạy đúng sau khi cài từ DMG
- [ ] (Nếu notarize) `spctl --assess` pass
- [ ] Updater artifacts (`latest.json`) được tạo

### B. Tham khảo

- [Tauri v2 — macOS Bundle](https://v2.tauri.app/distribute/macos/)
- [Tauri v2 — Code Signing](https://v2.tauri.app/distribute/sign/macos/)
- [Apple Developer — Notarizing](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
- [Apple Developer — Hardened Runtime](https://developer.apple.com/documentation/security/hardened-runtime)
- [create-dmg](https://github.com/create-dmg/create-dmg)

### C. Cấu trúc file liên quan trong dự án

```
proxypal/
├── src-tauri/
│   ├── Cargo.toml              # Rust dependencies + release profile
│   ├── tauri.conf.json         # Tauri config: bundle, signing, updater
│   ├── build.rs                # Build script (sidecar handling)
│   ├── Entitlements.plist      # (cần tạo) macOS entitlements
│   ├── binaries/               # Sidecar binaries (per-target)
│   └── icons/
│       └── icon.icns           # macOS app icon
├── vite.config.ts              # Frontend build config
├── package.json                # Node dependencies + scripts
├── .github/workflows/
│   └── release.yml             # CI/CD release workflow
└── scripts/
    └── build-macos.sh          # (khuyến nghị tạo) Local build script
```

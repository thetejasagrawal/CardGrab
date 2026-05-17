# Notarization setup

One-time setup so every tagged release is signed + notarized and opens without Gatekeeper warnings.

## What you need

- An **Apple Developer Program** account ($99/year — https://developer.apple.com/programs/)
- Xcode or Xcode Command Line Tools (`xcode-select --install`)
- About 15 minutes the first time

Once these are set up, the GitHub Action handles everything automatically on every tag push.

---

## Step 1 — Create the Developer ID Application certificate

1. Go to https://developer.apple.com/account/resources/certificates/list
2. Click the **+** to add a certificate
3. Pick **Developer ID Application** (not "Mac App Distribution")
4. Follow the CSR prompts (Keychain Access → Certificate Assistant → "Request a Certificate from a Certificate Authority")
5. Download the resulting `.cer` and **double-click** it to install into your login Keychain

Verify it's installed:

```bash
security find-identity -v -p codesigning | grep "Developer ID Application"
```

You should see a line like:

```
1) ABC1234567 "Developer ID Application: Tejas Agrawal (TEAMID12)"
```

The full string in quotes is your **APPLE_SIGNING_IDENTITY**. The `TEAMID12` part is your **APPLE_TEAM_ID** (also visible at https://developer.apple.com/account → Membership Details).

## Step 2 — Export the certificate as a `.p12`

Notarization in CI needs the cert + private key as a base64-encoded `.p12` file.

1. Open **Keychain Access** → **login** keychain → **My Certificates**
2. Find your "Developer ID Application: …" entry. Expand it — it must have a private key under it.
3. Right-click → **Export "Developer ID Application: …"** → save as `cert.p12`
4. Set an export password — anything strong. **Remember this password.**

Now base64-encode it:

```bash
base64 -i cert.p12 -o cert.p12.b64
pbcopy < cert.p12.b64
```

The contents of `cert.p12.b64` are your **APPLE_CERTIFICATE** secret. The export password is **APPLE_CERTIFICATE_PASSWORD**.

## Step 3 — Generate an app-specific password for notarytool

`notarytool` won't accept your regular Apple ID password.

1. Go to https://appleid.apple.com
2. Sign in → **Sign-In and Security** → **App-Specific Passwords**
3. Generate a new one labeled "cardgrab notarization"
4. Copy the password (looks like `abcd-efgh-ijkl-mnop`)

Your Apple ID email is **APPLE_ID**. The app-specific password is **APPLE_PASSWORD**.

## Step 4 — Add the six secrets to GitHub

Go to https://github.com/thetejasagrawal/CardGrab/settings/secrets/actions and add **Repository secrets**:

| Secret name                  | Value                                                        |
|------------------------------|--------------------------------------------------------------|
| `APPLE_CERTIFICATE`          | Contents of `cert.p12.b64` (from Step 2)                     |
| `APPLE_CERTIFICATE_PASSWORD` | The password you set when exporting (Step 2)                 |
| `APPLE_SIGNING_IDENTITY`     | `Developer ID Application: Your Name (TEAMID12)` (Step 1)    |
| `APPLE_ID`                   | Your Apple ID email                                          |
| `APPLE_PASSWORD`             | App-specific password from Step 3 (`abcd-efgh-ijkl-mnop`)    |
| `APPLE_TEAM_ID`              | Your team ID (e.g. `TEAMID12`)                               |

## Step 5 — Cut a release

```bash
git tag v0.1.0
git push --tags
```

The workflow at `.github/workflows/release.yml` will:

1. Build for both Apple Silicon (`aarch64`) and Intel (`x86_64`)
2. Sign each `.app` bundle with your Developer ID certificate
3. Submit each `.dmg` to Apple's notary service
4. Wait for notarization to complete (~2–10 minutes per arch)
5. Staple the notarization ticket onto the `.dmg`
6. Create a draft GitHub Release with both `.dmg`s attached

Find the draft at https://github.com/thetejasagrawal/CardGrab/releases — review, edit notes, **Publish**.

## Verification

After downloading the `.dmg` from a release:

```bash
xcrun stapler validate ~/Downloads/cardgrab_0.1.0_aarch64.dmg
# Should print: "The validate action worked!"

spctl -a -vvv -t install ~/Downloads/cardgrab_0.1.0_aarch64.dmg
# Should print: "accepted source=Notarized Developer ID"
```

If both succeed, double-clicking the `.dmg` and dragging into Applications will open without any Gatekeeper warning.

## Troubleshooting

- **"errSecInternalComponent" during signing** — usually means the cert imported in CI doesn't have its private key attached. Re-export the `.p12` and make sure the private key is included.
- **Notarization rejected with "The signature does not include a secure timestamp"** — Tauri handles this; if you see it, your Xcode tools might be out of date. `xcode-select --install`.
- **"Invalid main bundle identifier"** — the `identifier` in `src-tauri/tauri.conf.json` must match what's registered with App Store Connect. The current value is `com.cardgrab.app`; you can register it at https://developer.apple.com/account/resources/identifiers/list (App IDs → register a new identifier).
- **Workflow runs but skips signing** — one or more of the six secrets is missing or empty. Tauri's bundler logs `[skipping] signing requires…` when any of `APPLE_CERTIFICATE`, `APPLE_SIGNING_IDENTITY` are absent.
- **First-time team setup** — your Apple ID must be enrolled in the Apple Developer Program and have agreed to the latest Program License Agreement. Check at https://developer.apple.com/account.

# Ghostkeys

## Mac app

Build the app on a Mac with Node, Rust, and the Xcode command line tools:

```sh
npm ci
npm run build:mac
```

The app is at `src-tauri/target/release/bundle/macos/ghostkeys.app`.
The installer is in `src-tauri/target/release/bundle/dmg/`. Copy the app to
Applications before granting keyboard permission so macOS remembers the installed copy.
This build is signed locally; distributing it to other Macs requires Apple signing and notarization.

1. Open ghostkeys from Applications.
2. Use **Open System Settings** and enable ghostkeys under Privacy & Security → Accessibility
   (called **Device Control and Data Access** on newer macOS). If it is missing, use **+** to add the app from Applications.
3. Paste your text and click **Load Text**, or select a `.txt` or `.md` file.
4. Click **Start Typing**, then click the destination editor during the countdown.
5. Use **Control+Option+S** to stop from any app. Pause and resume are available in the widget and menu bar.

Ghostkeys types into the focused app on your Mac. Keep the destination editor focused while it runs.
The Mistakes settings can intentionally alter text; set the mistake rate to zero when exact text matters.

### Background typing on macOS

1. Click **Capture background editor** in Ghostkeys.
2. During the five-second countdown, click inside your destination document or text field.
3. Return to Ghostkeys, load your text, and start typing. You can now work in other apps.

Native editors use macOS Accessibility to edit the saved field directly. Arc and Helium receive
keystrokes sent only to the saved browser process. Keep the captured browser tab selected and its
document open; use a different app or browser while Ghostkeys types. Changing the destination's
text, caret, window, or tab stops typing. Background mode disables generated mistakes.
Editors that do not expose editable text to Accessibility cannot be captured. If an editor ignores
a character, Ghostkeys stops with an error rather than silently reporting completion.

TextEdit and disposable rich-text editors in Arc and Helium were tested with the destination app
in the background. Helium accepted Unicode, emoji, and paragraph breaks. Arc accepted accented
text, Greek characters, and paragraphs, but rejected emoji in this test. Word's web editor has not
been verified on this Mac; its support depends on the editor's Accessibility information.

For local automation, the existing loopback API also supports `capture_target` (optional `pid`)
and `clear_target`; `status` includes the saved target and whether its app is in the foreground.
The API stays disabled unless `GHOSTKEYS_API_TOKEN` is explicitly set.

## Ghostkeys on exe.dev

The desktop app runs continuously on the private `ghostkeys` VM, alongside Chrome.

- App: https://ghostkeys.exe.xyz/
- Desktop: https://ghostkeys.exe.xyz/desktop/vnc.html?autoconnect=1&resize=scale&path=desktop/websockify
- Separate Ghostkeys MCP: https://ghostkeys.exe.xyz/mcp
- Owner-only connection settings: https://ghostkeys.exe.xyz/connection
- Existing Waterloo MCP: https://discovery-brilliancy.exe.xyz/mcp

Sign in to exe.dev with the VM owner's account. The VM keeps running when you close the desktop tab. Microsoft is signed in with the Waterloo account. Microsoft can require another sign-in later.

## Use the app

1. Open your Word document in Chrome on the remote desktop.
2. Paste text into Ghostkeys or drop a plain text file onto it.
3. Press Start, then click where you want the text in Word during the countdown.
4. Use the app controls or Ctrl+Alt+S to stop.

Ghostkeys sends keys to the focused application **inside this VM**. Word must be open in the VM browser. The app does not type into your laptop's browser.

## Use both MCP connections

Ghostkeys has been added to the local Codex settings alongside Waterloo. Restart Codex to load the new connection. Other clients can copy the remote URL and custom header from the owner-only connection settings page. The generated Ghostkeys key expires after one year; replacing it also requires updating the server's client ID and the client settings.

Use Waterloo to read the requested school material. Use Ghostkeys to inspect the VM browser, open a Word URL, load your exact text, click the editor, and start typing. Neither connection submits coursework as part of this setup.

The Ghostkeys MCP has 12 tools: status, browser state, open Word, read tab, focus Word at the end, click, key, load text, start, pause, resume, and stop. Start uses the actual Rust typing engine. MCP typing disables generated mistakes to preserve the supplied text, checks that the target is a Word tab, and stops if Chrome loses focus or another tab becomes active. It may insert at the wrong caret if someone clicks elsewhere inside the same tab; inspect the screenshot and caret before starting, and avoid changing focus during typing. The manual desktop app types into whichever window has focus.

Every MCP start requires a new `actionId` for a new intended write. Repeating a recorded ID does not type again. If a write's outcome is unclear, read the document and typing status before requesting anything else. Word can autosave the typed text.

## Installed services

`ghostkeys-display`, `ghostkeys-window-manager`, `ghostkeys-vnc`, `ghostkeys-websocket`, `ghostkeys-chrome`, `ghostkeys-app`, `ghostkeys-mcp`, and `nginx` start at boot. Each Ghostkeys service restarts after an unexpected exit. Chrome's profile is stored at `/home/exedev/.config/ghostkeys/chrome`. A restarted Ghostkeys app starts empty; it does not resume partially typed text.

Nginx serves port 8000 through exe.dev's private HTTPS proxy. VNC, the Chrome debugging port, and the desktop control API listen only on loopback. The MCP key cannot access the remote desktop or the connection-key page. The desktop control API additionally requires a separate token stored in the VM's private service environment file.

On the VM:

```sh
systemctl is-active ghostkeys-display ghostkeys-window-manager ghostkeys-vnc ghostkeys-websocket ghostkeys-chrome ghostkeys-app ghostkeys-mcp nginx
sudo journalctl -u ghostkeys-app -u ghostkeys-mcp -n 100 --no-pager
sudo systemctl restart ghostkeys-app
```

For SSH from this Mac without modifying its existing SSH configuration:

```sh
ssh -i ~/.ssh/id_ed25519_exe_personal -o IdentitiesOnly=yes vm+ghostkeys@vm.exe.xyz
```

## Rebuild

The VM has Node 22, Rust, Chrome, and the Linux Tauri dependencies installed. Source lives at `/home/exedev/ghostkeys`.

```sh
export PATH=/home/exedev/.local/node/bin:/home/exedev/.cargo/bin:$PATH
cd /home/exedev/ghostkeys
npm ci
node scripts/generate-icons.mjs
npm run tauri build -- --no-bundle
cd deploy
npm ci
bash install-services.sh
sudo systemctl restart ghostkeys-app ghostkeys-mcp
```

Preserve `/home/exedev/.config/ghostkeys` when updating. It contains the Chrome profile, app token, and MCP client settings. The `deploy/mint-client.py` and `deploy/configure-codex.py` helpers are specific to this Mac and this VM. Do not run them for a different user without changing their paths and hostname.

## Verification on October 1, 2026

- Linux release build and TypeScript frontend build passed.
- Real Ghostkeys typed exact Unicode text and a paragraph break into a disposable Chrome editor.
- Pause, resume, stop, duplicate start, Unicode progress, and unauthenticated API rejection passed.
- Remote MCP initialization, 12-tool discovery, status, browser reads, and owner-page access restrictions passed.
- The app recovered after a service restart.
- With explicit owner approval, current Waterloo/Duo sign-in cookies were transferred to Chrome. No password or authenticator was copied. The temporary session transfer files were deleted.
- Microsoft accepted the saved school session. Ghostkeys then typed two paragraphs into a new Word document named **Ghostkeys setup test**. The app reached 100%, the Word screenshot showed both paragraphs in order, and Word showed the saved indicator.

The source changes are saved locally and deployed on the VM. They have not been pushed to GitHub.

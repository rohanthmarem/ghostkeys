# Ghostkeys for macOS

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

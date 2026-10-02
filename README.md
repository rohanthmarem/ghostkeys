# ghostkeys

A desktop typing simulator built with Tauri, React, and Rust. Load text, adjust typing speed and human-like pauses or mistakes, then type into the focused app. The floating progress widget, tray controls, and settings are shared by the Windows and macOS versions.

## Mac app (Apple Silicon, including M5)

The Mac build is a native ARM64 `.app` bundle. It does not require Rosetta, Node.js, or Rust to run. The deployment target is macOS 12 or later.

1. Copy `ghostkeys.app` to your **Applications** folder, then open it.
2. In **System Settings → Privacy & Security → Accessibility**, enable **ghostkeys**. If it is missing, use **+** to add the copy in Applications. macOS requires this permission before an app can type into another app.
3. Paste or write directly in the editor, or open/drop a text file. Choose Natural, Clean, or Quick; style and speed are in the bottom bar. Open Settings for delay, timing, and typo controls. Settings are saved automatically; drafts are saved locally when Remember draft is enabled.
4. Click **Start**, then focus the destination text field before the countdown ends. You can also focus the destination first and use **Control + Option + S** to start or stop. The same shortcut is **Ctrl + Alt + S** on Windows.

During the countdown, Cancel or Escape stops the session. Start/Stop and Pause/Resume remain in the bottom bar; the mini window opens automatically when typing starts and is also available from the header.

Resume also gives you a countdown (at least three seconds) to refocus the destination. Closing the main window hides it; use the menu bar icon's **Show Window** or click the Dock icon to reopen it. Use **Quit** to exit completely.

Keep the installed copy in the same location after granting permission. If you replace the app and macOS stops accepting keystrokes, remove its Accessibility entry, add the new app, and reopen it. In development, macOS may list the debug executable or the terminal running it separately from the installed app.

This personal build uses an ad-hoc signature and is not Apple-notarized. If a downloaded copy is blocked, try opening it once, then use **System Settings → Privacy & Security → Open Anyway** for that app. No system-wide security changes are needed.

## Build on a Mac

Install [Xcode Command Line Tools](https://v2.tauri.app/start/prerequisites/#macos), a current Node.js LTS version, and [Rust](https://www.rust-lang.org/tools/install). Then, from this repository:

```sh
npm ci
rustup target add aarch64-apple-darwin
npm run build:mac
```

The application is created at:

```text
src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ghostkeys.app
```

Open that bundle in Finder or copy it to Applications. The Mac configuration is automatically merged from `src-tauri/tauri.macos.conf.json`; it supplies Mac icons, enables the transparent widget, and packages an `.app` without changing Windows packaging.

The monochrome app mark is in `src-tauri/icons/icon.svg`. To regenerate the checked-in Mac icons after editing it:

```sh
npm run icons:mac
```

## Development

```sh
npm run dev          # Tauri app and Vite dev server
npm run dev:web      # Frontend preview only; native commands require Tauri
npm run build        # Type-check and build the frontend
cargo test --manifest-path src-tauri/Cargo.toml
```

The main settings window uses `src/App.tsx`; the floating widget uses `src/Widget.tsx`. Tauri commands and desktop integration live in `src-tauri/src/main.rs`. The typing engine, keyboard input, timing, and mistake generation live in `src-tauri/src/typer/`.

## Windows and automated builds

On Windows, use `npm ci` followed by `npm run build:tauri` with the [Windows Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows). The existing Windows release workflow remains in place.

The macOS workflow builds an Apple Silicon app for the `mac` branch, pull requests into `master`, or manual runs. Its downloadable ZIP preserves the `.app` bundle. It does not publish a release or notarize the app.

See Apple's [Accessibility permission instructions](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac) and Tauri's [macOS app bundle guide](https://v2.tauri.app/distribute/macos-application-bundle/) for platform details.

## Settings

- **Natural Slow** (42 WPM while typing) and **Natural Fast** (85 WPM) simulate drafting: several words flow together, then planning or rereading pauses; sentence and paragraph breaks take longer. Slow uses shorter bursts and more reflection. Overall output speed is lower than the WPM setting because pauses and revisions take time.
- Both drafting styles occasionally restart a partial word or replace a tentative word, then continue with your exact supplied text. All simulated typos are corrected, even if a saved custom correction rate is lower. This is a local rhythm simulation with a small set of tentative word alternatives, not an AI generating or understanding earlier drafts. Unicode text is preserved; revisions only erase newly typed ASCII. Destination-app autocorrect or formatting can still alter keystrokes.
- **Drafting rhythm** in Typing settings selects Slow, Fast, or Off independently of speed. In drafting modes, phrase-aware pauses replace the manual pause sliders. Existing settings and saved presets retain their previous behavior until a new rhythm is selected. Stopping during a revision leaves the partial draft in the destination; Stop never sends cleanup keystrokes.
- **Pause when switching apps** remembers the destination app at the end of the countdown. Switching apps pauses output. Return to that app and resume; the countdown gives you time to focus the text field again. This detects app changes, not a different field or window inside the same app.
- **Remember draft** restores your text after quitting. Turning it off removes the saved draft. Clear also clears the saved text.
- The **My presets** tab saves the current typing settings under a name; choose a saved preset to apply it or Delete to remove it.
- The **App** tab lets you record and apply separate start/stop and pause/resume combinations. Defaults are Control+Option+S and Control+Option+P.
- Select text in the editor to **Type selection**. Click in the editor to return to typing the full draft. Progress counts only the selected text.

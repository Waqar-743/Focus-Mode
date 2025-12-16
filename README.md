# Focus Timer

Pomodoro-based productivity desktop app built with **Tauri + Rust + React**.

## Download

Download the latest Windows installer from **GitHub Releases**:

- **Windows (recommended):** [Download FocusTimer-setup.exe](../../../releases/latest/download/FocusTimer-setup.exe)
- **Windows (MSI):** [Download FocusTimer.msi](../../../releases/latest/download/FocusTimer.msi)

Releases page:

[Latest release](../../../releases/latest)

## Features

- Work / short break / long break cycles (Pomodoro)
- Start / Pause / Reset controls
- System notifications on phase complete
- Settings (custom durations + daily goal) saved locally (SQLite)
- Today stats + streak

### Focus mode blocking (Windows)

During **Work** sessions, the app attempts to:

- Block: Facebook, YouTube, Twitter/X, Instagram, Reddit (via `hosts`)
- Suppress Windows toast notifications (mail/messages) via registry

**Note:** website blocking requires **Administrator** permissions on Windows. If you see an in-app warning banner, run the installed app **as Administrator**.

## Development

Install deps:

```bash
npm install
```

Run dev:

```bash
npm run tauri dev
```

Build installers:

```bash
npm run tauri build
```

Outputs:

- `src-tauri/target/release/bundle/nsis/*.exe`
- `src-tauri/target/release/bundle/msi/*.msi`

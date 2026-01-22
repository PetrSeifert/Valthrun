# Quick Start Guide

Get the GameBar ESP running in 5 minutes!

## Prerequisites

- Windows 10/11
- CS2 installed
- Visual Studio 2022 (for the widget)
- Rust nightly

## Step 1: Build the Controller

```bash
cd E:\Programming\Valthrun
cargo +nightly build --release -p controller-gamebar
```

Binary location: `target/release/controller-gamebar.exe`

## Step 2: Deploy the Game Bar Widget

1. Open `E:\Programming\overlay\overlay\overlay.sln` in Visual Studio
2. Set build configuration to **Debug** / **x64**
3. Right-click the `overlay` project → **Deploy**
4. Wait for deployment to complete

## Step 3: Start CS2

Launch CS2 and join a match (or deathmatch for quick testing)

## Step 4: Run the Controller

```bash
cd E:\Programming\Valthrun\target\release
.\controller-gamebar.exe
```

**Optional: Disable Encryption for Testing**
If you want to test without encryption (reduces latency):
```powershell
$env:OVERLAY_DISABLE_ENCRYPTION="true"
.\controller-gamebar.exe
```
Note: Make sure to use the unencrypted NetworkService in the widget (uncheck "Use Encryption").

You should see:
```
🎮 Valthrun GameBar Controller
================================

📡 Connecting to CS2...
✅ Connected to CS2 (PID: 12345)

🚀 Starting overlay server on 0.0.0.0:8082...
✅ Overlay server started!

📋 Waiting for GameBar overlay widget to connect...
   Connect your widget to: 127.0.0.1:8082
```

## Step 5: Open Game Bar

1. Press **Win+G** to open Xbox Game Bar
2. Click the **Widgets** menu (or press Win+G again to see all widgets)
3. Find and click the **overlay** widget
4. The widget should connect automatically

You should see in the controller:
```
✅ Widget connected!

🎯 Starting ESP...
```

## Step 6: See the ESP!

With Game Bar open (Win+G), you should now see:
- **Colored boxes** around enemy players (green/yellow/red based on health)
- **Blue boxes** around teammates
- **Health bars** on the left side of boxes

## Controls

- **Win+G** - Toggle Game Bar (and ESP visibility)
- **Ctrl+C** in controller terminal - Stop the controller

## Troubleshooting

### Enable Debug Logging

For detailed debugging, enable trace logs:

```powershell
$env:RUST_LOG="trace"
.\controller-gamebar.exe
```

This shows every draw call, player detected, and network operation!

### Widget won't connect

1. Check the widget is deployed: `Settings > Apps > overlay`
2. Restart Game Bar completely (close all widgets)
3. Check Windows Firewall isn't blocking port 8082

### No boxes visible

1. Make sure you're in a match with other players
2. Try deathmatch for immediate testing
3. Check the controller shows "Widget connected!"
4. Make sure Game Bar is open (Win+G)
5. **Enable trace logging** to see if players are being detected and drawn

### Controller can't find CS2

1. Launch CS2 first
2. Make sure CS2 is fully loaded (in menu or match)
3. Check Task Manager shows "cs2.exe" process

### Boxes are misaligned

This is a skeleton implementation - boxes are basic 2D rectangles. The original Valthrun controller has more sophisticated rendering.

## Next Steps

- Customize colors/settings in `controller-gamebar/src/settings.rs`
- Add more ESP features (see README.md)
- Try the interactive drawing example: `cargo run --example interactive_drawing` (from `overlay/rust-overlay-server/`)

## Tips

- Keep Game Bar pinned for persistent overlay
- The ESP runs at 60 FPS for smooth updates
- Close Game Bar (Win+G) to hide the ESP and play normally
- The widget will auto-reconnect if disconnected

Enjoy! 🎮


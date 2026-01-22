# Valthrun GameBar Controller

A lightweight CS2 ESP controller that uses the Xbox Game Bar overlay instead of a traditional render hook. This provides better compatibility and cleaner integration with Windows.

## Features

- **🎮 Game Bar Integration**: Uses the Xbox Game Bar overlay widget for rendering
- **🔒 No Render Hooks**: Doesn't hook into CS2's rendering pipeline
- **📡 Network-based**: Communicates with the overlay widget over encrypted TCP
- **⚡ Lightweight**: Minimal skeleton ESP implementation
- **🎯 Resolution Independent**: Drawing uses normalized coordinates (0.0-1.0)

## What's Included

This is a **skeleton ESP** with:
- ✅ Player bounding boxes (2D/3D)
- ✅ Health bars
- ✅ Team/enemy color coding
- ✅ Health-based color interpolation

## Requirements

1. **CS2** - Must be running before starting the controller
2. **Game Bar Overlay Widget** - The C# UWP widget from `overlay/` project
3. **Rust Nightly** - Required for cs2-schema features

## Building

From the Valthrun root directory:

```bash
cargo +nightly build --release -p controller-gamebar
```

The binary will be in `target/release/controller-gamebar.exe`

## Usage

### Step 1: Start the Controller

```bash
# Default (port 8082)
controller-gamebar.exe

# Custom port
controller-gamebar.exe --port 8083

# Disable encryption for testing (reduces latency)
$env:OVERLAY_DISABLE_ENCRYPTION="true"
controller-gamebar.exe
```

The controller will:
1. 🔍 Find and attach to the CS2 process
2. 🚀 Start the overlay server
3. ⏳ Wait for the widget to connect

### Step 2: Connect the Overlay Widget

1. **Deploy & Launch** the Game Bar widget (from Visual Studio)
2. **Open Game Bar** (Win+G)
3. **Open the Overlay Widget** from the widgets menu
4. The widget will automatically connect to `127.0.0.1:8082`

### Step 3: Play!

- Open Game Bar (Win+G) to see the ESP
- Close Game Bar to hide the ESP
- The ESP updates at 60 FPS

## How It Works

```
┌─────────────┐         ┌──────────────────┐
│ CS2 Memory  │◄────────│   Controller     │
│             │         │   (This App)     │
└─────────────┘         └────────┬─────────┘
                                 │ TCP
                                 │ (Encrypted)
                        ┌────────▼─────────┐
                        │  Game Bar Widget │
                        │   (C# UWP App)   │
                        └──────────────────┘
                                 │
                        ┌────────▼─────────┐
                        │  Game Bar Overlay│
                        │   (Over CS2)     │
                        └──────────────────┘
```

1. **Controller** reads CS2 memory using the existing Valthrun CS2 interface
2. **Transforms** 3D world coordinates to normalized 2D (0.0-1.0) coordinates  
3. **Sends** drawing commands over TCP to the widget
4. **Widget** renders the commands on the Game Bar overlay

## Configuration

Currently uses hardcoded default settings:

- **Enemies**: Green→Yellow→Red boxes based on health
- **Teammates**: Blue boxes
- **Health bars**: Enabled (left side)
- **Box type**: 2D rectangles

To customize, edit `src/settings.rs` and rebuild.

## Architecture Differences from Original Controller

| Original Controller | GameBar Controller |
|---------------------|-------------------|
| Vulkan render hook | Network-based rendering |
| ImGui overlay | XAML Game Bar widget |
| Direct rendering | Normalized coordinates |
| All ESP features | Skeleton ESP only |
| Full settings UI | Code-based settings |

## Limitations

This is a **skeleton implementation** demonstrating the concept. Missing features:

- ❌ Settings UI
- ❌ Skeleton rendering
- ❌ Head dots
- ❌ Weapon/distance info
- ❌ Text rendering (name, health, etc.)
- ❌ Filled shapes
- ❌ Config file support
- ❌ Other enhancements (aim assist, bomb timer, etc.)

## Future Improvements

- Add UI control panel using the overlay-server's UI system
- Implement text rendering in the protocol
- Add filled shapes support
- More ESP features (skeleton, snaplines, etc.)
- Configuration file support
- Dynamic settings via UI

## Debug Logging

To see detailed trace logs of every draw call, set the `RUST_LOG` environment variable:

```bash
# Windows PowerShell
$env:RUST_LOG="trace"
controller-gamebar.exe

# Windows CMD
set RUST_LOG=trace
controller-gamebar.exe
```

This will show:
- Every `render_esp()` call
- Number of players found
- Each player being rendered (team, health, position)
- Every draw function call (`draw_rectangle`, `draw_line`, etc.)
- Network broadcasts to widgets
- Individual send confirmations to each client

Example trace output:
```
TRACE controller_gamebar: render_esp() called
TRACE controller_gamebar: Found 5 players to render
TRACE controller_gamebar: Rendering player 1/5: team=2, health=100, is_enemy=true
TRACE controller_gamebar:   Box 2D calculated: min=(0.450, 0.300), max=(0.550, 0.700)
TRACE overlay_server: draw_rectangle(0.450, 0.300, w=0.100, h=0.400, #00FF00, line_w=0.002)
TRACE overlay_server: broadcast to 1 client(s): 89 bytes
TRACE overlay_server:   -> sent to client 1/1 (127.0.0.1:52431)
```

## Troubleshooting

**"Failed to create CS2 handle. Is CS2 running?"**
- Launch CS2 before starting the controller

**"Widget disconnected"**
- Make sure the Game Bar widget is deployed and running
- Check the port matches (default 8082)

**"No ESP visible"**
- Open Game Bar with Win+G
- Make sure you're in a match with other players
- Check if widget is connected (controller will log connection)
- Enable trace logging to see if draw calls are being made

## Development

The codebase is organized as:

- `main.rs` - Main loop, CS2 reading, network comm
- `view_controller.rs` - 3D→2D coordinate transformation
- `settings.rs` - ESP configuration structures

Key dependencies:
- `cs2` - CS2 memory reading
- `overlay-server` - Network protocol and drawing commands
- `utils-state` - State management system

## License

Part of the Valthrun project - see main LICENSE file.


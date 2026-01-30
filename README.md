# WenPlayer - FFmpeg-based Audio Player

A cross-platform audio player built with Tauri, Rust, and FFmpeg.

## Features

- **FFmpeg Audio Decoding**: Support for a wide range of audio formats (MP3, WAV, FLAC, OGG, M4A, etc.)
- **CPAL Audio Output**: Low-latency audio playback using CPAL
- **Playback Controls**: Play, pause, stop, seek, and volume control
- **Real-time Status**: Monitor playback state, current time, and duration
- **Cross-platform**: Works on Windows, macOS, and Linux

## System Dependencies

### Linux

You need to install the complete C build toolchain and FFmpeg development libraries:

```bash
# Ubuntu/Debian
sudo apt-get update

# Install complete build toolchain + FFmpeg + ALSA
sudo apt-get install -y \
    build-essential \
    libc6-dev \
    gcc-multilib \
    g++-multilib \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswresample-dev \
    libswscale-dev \
    pkg-config \
    libasound2-dev \
    clang \
    libclang-dev
```

**Note**: The error about `limits.h` is resolved by installing `libc6-dev` (included in build-essential).

### macOS

Install FFmpeg via Homebrew and Xcode command line tools:
```bash
brew install ffmpeg
```

### macOS

Install FFmpeg via Homebrew:
```bash
brew install ffmpeg
```

### macOS

No additional system dependencies required.

### Windows

No additional system dependencies required.

## Development Setup

1. Install Node.js dependencies:
```bash
pnpm install
```

2. Install Rust toolchain (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. Install rust-analyzer for VS Code:
- Install from VS Code Marketplace: [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- Or via VS Code extensions tab: Search for "rust-analyzer"

## Available Tauri Commands

### Core Commands

- `load_file(path: string)` - Load an audio file
- `play()` - Start or resume playback
- `pause()` - Pause playback
- `stop()` - Stop playback and reset position
- `toggle_playback()` - Toggle between play and pause

### Control Commands

- `seek_to(position: number)` - Seek to position in seconds (0.0 to duration)
- `set_volume(volume: number)` - Set volume (0.0 to 1.0)
- `get_player_status()` - Get current player status

### Status Returns

The `get_player_status()` command returns:
```typescript
{
  is_playing: boolean,
  current_time: number,
  duration: number,
  volume: number,
  file_path: string | null
}
```

## Building

### Development Mode
```bash
pnpm tauri dev
```

### Production Build
```bash
pnpm tauri build
```

## Project Structure

- `src-tauri/src/` - Rust backend code
  - `lib.rs` - Main entry point and Tauri commands
  - `decoder.rs` - FFmpeg audio decoder
  - `audio_output.rs` - CPAL audio output
  - `player.rs` - Main player implementation
- `src/` - Frontend TypeScript/HTML/CSS code

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

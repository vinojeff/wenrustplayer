// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod decoder;
mod audio_output;
mod player;

use crossbeam_channel::unbounded;
use decoder::VideoFrame;
use player::{MediaPlayer, PlayerStatus, PlaybackState};
use tauri::{State, Emitter};
use std::sync::Mutex;

/// Global player instance
type SharedPlayer = Mutex<MediaPlayer>;

// Default greeting command (kept for reference)
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Load a media file
#[tauri::command]
async fn load_file(
    path: String, 
    player: State<'_, SharedPlayer>,
    app_handle: tauri::AppHandle
) -> Result<PlayerStatus, String> {
    let mut p = player.lock().unwrap();
    
    // Create video frame channel for sending frames to frontend
    let (video_sender, video_receiver) = unbounded::<VideoFrame>();
    
    // Load the file with video sender
    let status = p.load(&path, Some(video_sender))
        .map_err(|e| format!("Failed to load file: {}", e))?;
    
    // Start video frame emitter thread
    std::thread::spawn(move || {
        while let Ok(frame_data) = video_receiver.recv() {
            // Emit video frame to frontend
            let _ = app_handle.emit("video-frame", frame_data);
        }
    });
    
    Ok(status)
}

/// Play the media
#[tauri::command]
async fn play(player: State<'_, SharedPlayer>) -> Result<(), String> {
    let mut p = player.lock().unwrap();
    p.play().map_err(|e| format!("Failed to play: {}", e))
}

/// Pause the media
#[tauri::command]
async fn pause(player: State<'_, SharedPlayer>) -> Result<(), String> {
    let mut p = player.lock().unwrap();
    p.pause().map_err(|e| format!("Failed to pause: {}", e))
}

/// Toggle playback (play/pause)
#[tauri::command]
async fn toggle_playback(player: State<'_, SharedPlayer>) -> Result<bool, String> {
    let mut p = player.lock().unwrap();
    let is_playing = p.get_state() == PlaybackState::Playing;

    if is_playing {
        p.pause().map_err(|e| format!("Failed to pause: {}", e))?;
        Ok(false)
    } else {
        p.play().map_err(|e| format!("Failed to play: {}", e))?;
        Ok(true)
    }
}

/// Stop playback
#[tauri::command]
async fn stop(player: State<'_, SharedPlayer>) -> Result<(), String> {
    let mut p = player.lock().unwrap();
    p.stop();
    Ok(())
}

/// Seek to a specific time in seconds
#[tauri::command]
async fn seek_to(position: f64, player: State<'_, SharedPlayer>) -> Result<f64, String> {
    let mut p = player.lock().unwrap();
    p.seek(position).map_err(|e| format!("Failed to seek: {}", e))?;
    Ok(position)
}

/// Set volume (0.0 - 1.0)
#[tauri::command]
async fn set_volume(volume: f32, player: State<'_, SharedPlayer>) -> Result<f32, String> {
    let mut p = player.lock().unwrap();
    p.set_volume(volume);
    Ok(p.get_volume())
}

/// Get the current player status
#[tauri::command]
async fn get_player_status(player: State<'_, SharedPlayer>) -> Result<PlayerStatus, String> {
    let p = player.lock().unwrap();
    Ok(p.get_status())
}

/// Previous track (placeholder for playlist support)
#[tauri::command]
async fn previous_track() -> Result<(), String> {
    println!("Previous track requested");
    Ok(())
}

/// Next track (placeholder for playlist support)
#[tauri::command]
async fn next_track() -> Result<(), String> {
    println!("Next track requested");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player = Mutex::new(MediaPlayer::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(player)
        .invoke_handler(tauri::generate_handler![
            greet,
            load_file,
            play,
            pause,
            toggle_playback,
            stop,
            seek_to,
            set_volume,
            get_player_status,
            previous_track,
            next_track
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

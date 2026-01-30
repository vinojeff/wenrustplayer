import {
  getCurrentWindow,
} from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// 获取当前窗口
const currentWindow = getCurrentWindow();

// 视频画布上下文
let videoCanvas: HTMLCanvasElement | null = null;
let videoContext: CanvasRenderingContext2D | null = null;

// 视频帧数据结构
interface VideoFrameData {
  width: number;
  height: number;
  data: number[]; // RGBA bytes
  timestamp: number;
}

// 渲染视频帧
function renderVideoFrame(frameData: VideoFrameData) {
  if (!videoCanvas || !videoContext) {
    videoCanvas = document.getElementById('video-canvas') as HTMLCanvasElement;
    if (videoCanvas) {
      videoContext = videoCanvas.getContext('2d');
    }
  }

  if (!videoCanvas || !videoContext) {
    console.error('视频画布未找到');
    return;
  }

  // 设置画布尺寸（只在尺寸变化时）
  if (videoCanvas.width !== frameData.width || videoCanvas.height !== frameData.height) {
    videoCanvas.width = frameData.width;
    videoCanvas.height = frameData.height;
  }

  // 创建 ImageData 并绘制
  const imageData = new ImageData(
    new Uint8ClampedArray(frameData.data),
    frameData.width,
    frameData.height
  );
  
  videoContext.putImageData(imageData, 0, 0);
}

// 窗口控制功能
async function minimizeWindow() {
  await currentWindow.minimize();
}

async function maximizeWindow() {
  await currentWindow.toggleMaximize();
}

async function closeWindow() {
  await currentWindow.close();
}

// 文件打开功能
async function openFileDialog() {
  console.log("打开文件对话框被调用...");
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "所有文件",
          extensions: ["*"],
        },
        {
          name: "视频",
          extensions: ["mp4", "mkv", "avi", "mov", "wmv", "flv"],
        },
        {
          name: "音频",
          extensions: ["mp3", "flac", "wav", "aac", "ogg"],
        },
      ],
    });

    console.log("文件对话框返回:", selected);

    if (selected && typeof selected === "string") {
      const fileName = selected.split(/[/\\]/).pop() || selected;
      const fileInfoEl = document.getElementById("file-info");
      const fileNameEl = document.getElementById("file-name");

      if (fileInfoEl && fileNameEl) {
        fileNameEl.textContent = `已打开: ${fileName}`;
        fileInfoEl.classList.remove("hidden");
      }
      console.log("打开文件:", selected);

      // 最大化窗口
      await maximizeWindow();

      // 调用 Rust 加载文件
      try {
        await invoke('load_file', { path: selected });
        console.log('文件加载到播放器成功');
      } catch (error) {
        console.error('加载文件到播放器失败:', error);
        // 继续显示 UI，但播放可能不可用
      }

      // 显示对应的媒体图标
      showMediaIcon(selected);
    } else {
      console.log("用户取消了文件选择或返回了无效值");
    }
  } catch (error) {
    console.error("打开文件时出错:", error);
    alert(`打开文件失败: ${error}`);
  }
}

async function openFolderDialog() {
  console.log("打开文件夹对话框被调用...");
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    console.log("文件夹对话框返回:", selected);

    if (selected && typeof selected === "string") {
      const folderName = selected.split(/[/\\]/).pop() || selected;
      const fileInfoEl = document.getElementById("file-info");
      const fileNameEl = document.getElementById("file-name");

      if (fileInfoEl && fileNameEl) {
        fileNameEl.textContent = `已打开文件夹: ${folderName}`;
        fileInfoEl.classList.remove("hidden");
      }
      console.log("打开文件夹:", selected);
    } else {
      console.log("用户取消了文件夹选择或返回了无效值");
    }
  } catch (error) {
    console.error("打开文件夹时出错:", error);
    alert(`打开文件夹失败: ${error}`);
  }
}

// 菜单切换
function toggleMenu() {
  const dropdown = document.getElementById("file-dropdown");
  if (dropdown) {
    dropdown.classList.toggle("show");
  }
}

function closeMenu() {
  const dropdown = document.getElementById("file-dropdown");
  if (dropdown) {
    dropdown.classList.remove("show");
  }
}

// 显示媒体图标和播放器控件
function showMediaIcon(filePath: string) {
  const extension = filePath.split('.').pop()?.toLowerCase() || '';
  const mediaPlayer = document.getElementById('media-player');
  const audioIcon = document.getElementById('audio-icon');
  const videoIcon = document.getElementById('video-icon');
  const videoCanvas = document.getElementById('video-canvas') as HTMLCanvasElement;
  const playerControls = document.getElementById('player-controls');

  // 音频文件扩展名
  const audioExtensions = ['mp3', 'flac', 'wav', 'aac', 'ogg', 'm4a', 'wma'];
  // 视频文件扩展名
  const videoExtensions = ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v'];

  // 隐藏所有图标和控件
  if (audioIcon) audioIcon.style.display = 'none';
  if (videoIcon) videoIcon.style.display = 'none';
  if (videoCanvas) videoCanvas.style.display = 'none';
  if (playerControls) playerControls.style.display = 'none';

  // 根据文件类型显示对应界面
  if (audioExtensions.includes(extension)) {
    if (audioIcon) audioIcon.style.display = 'block';
    console.log('显示音频图标');
    // 显示播放器控件
    if (playerControls) playerControls.style.display = 'flex';
  } else if (videoExtensions.includes(extension)) {
    if (videoIcon) videoIcon.style.display = 'block';
    console.log('显示视频图标');
    // 初始化视频画布
    if (videoCanvas) {
      videoCanvas.style.display = 'block';
      // 设置画布尺寸（示例：根据窗口调整）
      videoCanvas.width = videoCanvas.clientWidth;
      videoCanvas.height = videoCanvas.clientHeight;
    }
    // 显示播放器控件
    if (playerControls) playerControls.style.display = 'flex';
  } else {
    // 默认显示音频图标
    if (audioIcon) audioIcon.style.display = 'block';
    console.log('显示默认图标');
    // 显示播放器控件
    if (playerControls) playerControls.style.display = 'flex';
  }

  // 显示播放器容器
  if (mediaPlayer) {
    mediaPlayer.classList.add('active');
  }

  // 切换到播放器模式
  document.body.classList.add('player-mode');
}

// 播放器状态
let isPlaying = false;
let currentTime = 0;
let duration = 0;
let volume = 0.8;

// 切换播放/暂停
async function togglePlayPause() {
  const playPauseBtn = document.getElementById('play-pause-btn');
  const playIcon = document.getElementById('play-icon');
  const pauseIcon = document.getElementById('pause-icon');
  
  try {
    // 调用 Rust 命令
    const playing = await invoke<boolean>('toggle_playback');
    isPlaying = playing;
    
    if (playPauseBtn && playIcon && pauseIcon) {
      if (isPlaying) {
        playIcon.style.display = 'none';
        pauseIcon.style.display = 'block';
        playPauseBtn.setAttribute('aria-label', '暂停');
        console.log('开始播放');
      } else {
        playIcon.style.display = 'block';
        pauseIcon.style.display = 'none';
        playPauseBtn.setAttribute('aria-label', '播放');
        console.log('暂停播放');
      }
    }
  } catch (error) {
    console.error('切换播放状态失败:', error);
    // 前端回退
    isPlaying = !isPlaying;
    updatePlayPauseUI();
  }
}

// 更新播放/暂停UI
function updatePlayPauseUI() {
  const playPauseBtn = document.getElementById('play-pause-btn');
  const playIcon = document.getElementById('play-icon');
  const pauseIcon = document.getElementById('pause-icon');
  
  if (playPauseBtn && playIcon && pauseIcon) {
    if (isPlaying) {
      playIcon.style.display = 'none';
      pauseIcon.style.display = 'block';
      playPauseBtn.setAttribute('aria-label', '暂停');
    } else {
      playIcon.style.display = 'block';
      pauseIcon.style.display = 'none';
      playPauseBtn.setAttribute('aria-label', '播放');
    }
  }
}

// 更新进度条
function updateProgress(percent: number) {
  const progressFill = document.getElementById('progress-fill');
  if (progressFill) {
    progressFill.style.width = `${percent}%`;
  }
}

// 更新时间显示
function updateTimeDisplay(current: number, total: number) {
  const currentTimeEl = document.getElementById('current-time');
  const durationEl = document.getElementById('duration');
  
  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };
  
  if (currentTimeEl) {
    currentTimeEl.textContent = formatTime(current);
  }
  if (durationEl) {
    durationEl.textContent = formatTime(total);
  }
}

// 处理进度条点击
function setupProgressBar() {
  const progressBar = document.getElementById('progress-bar');
  if (!progressBar) return;
  
  progressBar.addEventListener('click', async (e) => {
    const rect = progressBar.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percent = (x / rect.width) * 100;
    updateProgress(percent);
    
    // 计算时间
    const newTime = (percent / 100) * duration;
    
    try {
      // 调用 Rust 跳转命令
      const seekedTime = await invoke<number>('seek_to', { position: newTime });
      currentTime = seekedTime;
      console.log(`跳转到: ${percent.toFixed(1)}% (${currentTime.toFixed(1)}秒)`);
    } catch (error) {
      console.error('跳转失败:', error);
      currentTime = newTime;
    }
    
    updateTimeDisplay(currentTime, duration);
  });
}

// 设置音量控制
function setupVolumeControl() {
  const volumeSlider = document.getElementById('volume-slider') as HTMLInputElement;
  const volumeBtn = document.getElementById('volume-btn');
  const volumeHighIcon = document.getElementById('volume-high-icon');
  const volumeLowIcon = document.getElementById('volume-low-icon');
  const volumeMuteIcon = document.getElementById('volume-mute-icon');
  
  if (!volumeSlider || !volumeBtn || !volumeHighIcon || !volumeLowIcon || !volumeMuteIcon) return;
  
  // 音量滑块变化
  volumeSlider.addEventListener('input', async (e) => {
    const target = e.target as HTMLInputElement;
    const newVolume = parseInt(target.value) / 100;
    
    try {
      // 调用 Rust 音量设置命令
      const actualVolume = await invoke<number>('set_volume', { volume: newVolume });
      volume = actualVolume;
      console.log(`音量设置: ${volume}`);
    } catch (error) {
      console.error('设置音量失败:', error);
      volume = newVolume;
    }
    
    // 更新音量图标
    if (volume === 0) {
      volumeHighIcon.style.display = 'none';
      volumeLowIcon.style.display = 'none';
      volumeMuteIcon.style.display = 'block';
    } else if (volume < 0.5) {
      volumeHighIcon.style.display = 'none';
      volumeLowIcon.style.display = 'block';
      volumeMuteIcon.style.display = 'none';
    } else {
      volumeHighIcon.style.display = 'block';
      volumeLowIcon.style.display = 'none';
      volumeMuteIcon.style.display = 'none';
    }
  });
  
  // 音量按钮点击静音/恢复
  volumeBtn.addEventListener('click', async () => {
    let newVolume = volume;
    
    if (volume > 0) {
      // 保存当前音量并静音
      (window as any).lastVolume = volume;
      newVolume = 0;
    } else {
      // 恢复音量
      newVolume = (window as any).lastVolume || 0.8;
    }
    
    try {
      // 调用 Rust 音量设置命令
      const actualVolume = await invoke<number>('set_volume', { volume: newVolume });
      volume = actualVolume;
      console.log(`音量: ${volume > 0 ? '恢复' : '静音'}`);
    } catch (error) {
      console.error('设置音量失败:', error);
      volume = newVolume;
    }
    
    // 更新 UI
    volumeSlider.value = (volume * 100).toString();
    if (volume === 0) {
      volumeHighIcon.style.display = 'none';
      volumeLowIcon.style.display = 'none';
      volumeMuteIcon.style.display = 'block';
    } else if (volume < 0.5) {
      volumeHighIcon.style.display = 'none';
      volumeLowIcon.style.display = 'block';
      volumeMuteIcon.style.display = 'none';
    } else {
      volumeHighIcon.style.display = 'block';
      volumeLowIcon.style.display = 'none';
      volumeMuteIcon.style.display = 'none';
    }
  });
}

// 初始化播放器控件
function initPlayerControls() {
  setupProgressBar();
  setupVolumeControl();
  
  // 示例数据：模拟一个3分钟的音频
  duration = 180; // 3分钟 = 180秒
  updateTimeDisplay(currentTime, duration);
  updateProgress(0);
  
  // 模拟播放进度（仅用于演示）
  // 在实际应用中，这应该由后端驱动
  setInterval(() => {
    if (isPlaying && currentTime < duration) {
      currentTime += 1;
      const percent = (currentTime / duration) * 100;
      updateProgress(percent);
      updateTimeDisplay(currentTime, duration);
    }
  }, 1000);
}

// 初始化事件监听
window.addEventListener("DOMContentLoaded", () => {
  // 窗口控制按钮
  document.getElementById("minimize-btn")?.addEventListener("click", minimizeWindow);
  document.getElementById("maximize-btn")?.addEventListener("click", maximizeWindow);
  document.getElementById("close-btn")?.addEventListener("click", closeWindow);

  // 文件菜单按钮
  document.getElementById("file-menu-btn")?.addEventListener("click", (e) => {
    e.stopPropagation();
    toggleMenu();
  });

  // 菜单项
  document.getElementById("open-file-btn")?.addEventListener("click", (e) => {
    e.stopPropagation();
    console.log("点击了打开文件按钮");
    openFileDialog();
    closeMenu();
  });

  document.getElementById("open-folder-btn")?.addEventListener("click", (e) => {
    e.stopPropagation();
    console.log("点击了打开文件夹按钮");
    openFolderDialog();
    closeMenu();
  });

  document.getElementById("exit-btn")?.addEventListener("click", (e) => {
    e.stopPropagation();
    closeWindow();
    closeMenu();
  });

  // 主界面打开按钮
  document.getElementById("main-open-btn")?.addEventListener("click", () => {
    console.log("点击了主界面打开文件按钮");
    openFileDialog();
  });

  // 播放器控制按钮
  document.getElementById("play-pause-btn")?.addEventListener("click", togglePlayPause);
  document.getElementById("prev-btn")?.addEventListener("click", async () => {
    console.log("上一首");
    try {
      await invoke('previous_track');
    } catch (error) {
      console.error('上一首失败:', error);
    }
  });
  document.getElementById("next-btn")?.addEventListener("click", async () => {
    console.log("下一首");
    try {
      await invoke('next_track');
    } catch (error) {
      console.error('下一首失败:', error);
    }
  });

  // 初始化播放器控件
  initPlayerControls();

  // 设置视频帧监听
  listen<VideoFrameData>('video-frame', (event) => {
    const frameData = event.payload;
    renderVideoFrame(frameData);
  }).then(() => {
    console.log('视频帧监听器已设置');
  }).catch((err) => {
    console.error('设置视频帧监听器失败:', err);
  });

  // 点击其他地方关闭菜单
  document.addEventListener("click", (e) => {
    const menuDropdown = document.querySelector(".menu-dropdown");
    if (menuDropdown && !menuDropdown.contains(e.target as Node)) {
      closeMenu();
    }
  });

  // 监听窗口最大化状态变化，更新图标
  getCurrentWindow().onResized(async () => {
    const maximizeBtn = document.getElementById("maximize-btn");
    if (maximizeBtn) {
      const isMaximized = await currentWindow.isMaximized();
      if (isMaximized) {
        maximizeBtn.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="4" y="4" width="16" height="16" rx="2" ry="2"></rect>
          </svg>
        `;
      } else {
        maximizeBtn.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          </svg>
        `;
      }
    }
  });
});

console.log("WenPlayer initialized successfully!");

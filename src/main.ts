import {
  getCurrentWindow,
} from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";

// 获取当前窗口
const currentWindow = getCurrentWindow();

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

// 显示媒体图标
function showMediaIcon(filePath: string) {
  const extension = filePath.split('.').pop()?.toLowerCase() || '';
  const mediaPlayer = document.getElementById('media-player');
  const audioIcon = document.getElementById('audio-icon');
  const videoIcon = document.getElementById('video-icon');

  // 音频文件扩展名
  const audioExtensions = ['mp3', 'flac', 'wav', 'aac', 'ogg', 'm4a', 'wma'];
  // 视频文件扩展名
  const videoExtensions = ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v'];

  // 隐藏所有图标
  if (audioIcon) audioIcon.style.display = 'none';
  if (videoIcon) videoIcon.style.display = 'none';

  // 根据文件类型显示对应图标
  if (audioExtensions.includes(extension)) {
    if (audioIcon) audioIcon.style.display = 'block';
    console.log('显示音频图标');
  } else if (videoExtensions.includes(extension)) {
    if (videoIcon) videoIcon.style.display = 'block';
    console.log('显示视频图标');
  } else {
    // 默认显示音频图标
    if (audioIcon) audioIcon.style.display = 'block';
    console.log('显示默认图标');
  }

  // 显示播放器容器
  if (mediaPlayer) {
    mediaPlayer.classList.add('active');
  }

  // 切换到播放器模式
  document.body.classList.add('player-mode');
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

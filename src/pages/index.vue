<script setup>
import { ref, computed, onMounted } from "vue";
import {
  Refresh,
  Top,
  Remove,
  Monitor,
  Star,
  Setting,
  Aim,
  Cpu,
  CircleCheck,
  Minus,
  Close,
} from "@element-plus/icons-vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { invoke, Channel } from "@tauri-apps/api/core";
import { APP_MAIN_WINDOW_LABEL } from "@/common/constant";
import { listen } from "@tauri-apps/api/event";

listen("check_update", (event) => {
  fetch_update();
});

listen("sys_error", (event) => {
  ElMessage({
    showClose: true,
    message: event.payload,
    type: "error",
  });
});
const onEvent = new Channel();
const fetch_update = () => {
  invoke("fetch_update", {})
    .then((res) => {
      if (res) {
        ElMessageBox.confirm("存在新版本，确认要更新吗?", "系统提示", {
          confirmButtonText: "确定",
          cancelButtonText: "取消",
          type: "info",
        }).then(() => {
          invoke("install_update", { onEvent: onEvent });
        });
      }
    })
    .catch((e) => {
      console.error(e);
      ElMessage({
        showClose: true,
        message: e,
        type: "error",
      });
    });
};

const loading = ref(false);
const windowList = ref([]);
const refreshing = ref(false);
const scrollbarHeight = ref(`${window.innerHeight - 70}px`);

// 计算属性
const topMostCount = computed(() => {
  return windowList.value.filter((w) => w.is_top_most).length;
});

// 初始化列表
const loadWindows = (msg) => {
  loading.value = true;
  invoke("list_windows")
    .then((data) => {
      windowList.value = data;
      if (msg) {
        ElMessage({
          showClose: true,
          message: msg,
          type: "success",
        });
      }
    })
    .catch((err) => {
      console.log("load windows error", err);
    })
    .finally(() => {
      loading.value = false;
    });
};

// 刷新列表
const refreshList = () => {
  loadWindows("窗口列表已刷新");
};

// 切换置顶状态
const toggleTopMost = (window) => {
  loading.value = true;
  window.is_top_most = !window.is_top_most;
  invoke("toggle_topmost", {
    hwnd: window.hwnd,
    enable: window.is_top_most,
  })
    .then(() => {
      const action = window.is_top_most ? "置顶" : "取消置顶";
      ElMessage({
        showClose: true,
        message: `${window.display_name} ${action}成功`,
        type: "success",
      });
    })
    .catch((err) => {
      console.log("toggle topmost error", err);
    })
    .finally(() => {
      loading.value = false;
    });
};

// 取消所有置顶
const cancelAllTopMost = () => {
  loading.value = true;
  let cancelTopMostList = [];
  windowList.value.forEach((window) => {
    if (window.is_top_most) {
      window.is_top_most = false;
      cancelTopMostList.push(window.hwnd);
    }
  });
  // 调用API取消置顶
  invoke("cancel_all_topmost", { hwndList: cancelTopMostList })
    .then(() => {
      ElMessage({
        showClose: true,
        message: "已取消所有窗口置顶",
        type: "success",
      });
    })
    .catch((err) => {
      console.log("cancel all topmost error", err);
    })
    .finally(() => {
      loading.value = false;
    });
};

// 关闭窗口
const closeWindow = () => {
  invoke("close_window", { windowLabel: APP_MAIN_WINDOW_LABEL }).catch(
    (err) => {
      console.log("close window error", err);
    }
  );
};

const minimizeWindow = () => {
  invoke("minimize_window", { windowLabel: APP_MAIN_WINDOW_LABEL }).catch(
    (err) => {
      console.log("minimize window error", err);
    }
  );
};

onMounted(() => {
  loadWindows();

  onEvent.onmessage = (message) => {
    if (message.event == "Started") {
      that.updateSize = message.data.contentLength;
      that.updateCurrentSize = 0;
      that.updateProgress = 0;
      that.updateDialogVisible = true;
    } else if (message.event == "Progress") {
      that.updateCurrentSize += message.data.chunkLength;
      that.updateProgress = Math.floor(
        (that.updateCurrentSize / that.updateSize) * 100
      );
    } else if (message.event == "Finished") {
      that.updateProgress = 100;
      setTimeout(() => {
        that.updateDialogVisible = false;
      }, 500);
    } else if (message.event == "sys_error") {
      console.log("sys_error:", message.data);
    }
  };

  fetch_update();
});
</script>

<template>
  <div
    class="topmost-manager"
    v-loading="loading"
    element-loading-background="rgba(255, 255, 255, 0.5)"
  >
    <!-- 标题栏 -->
    <div class="header">
      <div class="header-left">
        <el-icon size="18" color="#409EFF"><Monitor /></el-icon>
        <h3>窗口管理器</h3>
      </div>
      <div class="header-actions">
        <el-tooltip content="刷新窗口列表" placement="top">
          <el-button
            type="primary"
            :icon="Refresh"
            circle
            size="small"
            @click="refreshList"
            :loading="refreshing"
          />
        </el-tooltip>
        <!-- <el-tooltip content="最小化" placement="top"> -->
        <el-button
          type="primary"
          :icon="Minus"
          circle
          size="small"
          @click="minimizeWindow"
        />
        <!-- </el-tooltip> -->
        <!-- <el-tooltip content="关闭" placement="top"> -->
        <el-button
          type="primary"
          :icon="Close"
          circle
          size="small"
          @click="closeWindow"
        />
        <!-- </el-tooltip> -->
      </div>
    </div>

    <!-- 窗口列表 -->
    <el-scrollbar :height="scrollbarHeight">
      <div class="window-list">
        <div
          v-for="window in windowList"
          :key="window.id"
          :class="['window-item', { 'is-topmost': window.is_top_most }]"
          @dblclick="focusWindow(window)"
        >
          <!-- 程序图标 -->
          <div class="window-icon">
            <div class="icon-wrapper">
              <img
                v-if="window.icon_png"
                :src="`data:image/png;base64,${window.icon_png}`"
                :alt="window.display_name"
                class="app-icon"
              />
              <el-icon v-else class="default-icon">
                <Monitor />
              </el-icon>
              <div v-if="window.isActive" class="active-indicator"></div>
            </div>
          </div>

          <!-- 窗口信息 -->
          <div class="window-info">
            <div class="app-name-row">
              <span class="app-name" :title="window.title">{{
                window.title
              }}</span>
              <div class="status-badges">
                <el-tag
                  v-if="window.is_top_most"
                  type="primary"
                  size="small"
                  effect="dark"
                  class="topmost-badge"
                >
                  置顶
                </el-tag>
              </div>
            </div>

            <div class="window-meta">
              <span class="process-info">
                <el-icon size="10"><Cpu /></el-icon>
                {{ window.display_name }}
              </span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="window-actions">
            <el-tooltip
              :content="window.is_top_most ? '取消置顶' : '置顶显示'"
              placement="top"
            >
              <el-button
                :type="window.is_top_most ? 'info' : 'primary'"
                :icon="window.is_top_most ? Remove : Top"
                circle
                size="small"
                @click.stop="toggleTopMost(window)"
                class="topmost-btn"
              />
            </el-tooltip>
          </div>
        </div>
      </div>
    </el-scrollbar>

    <!-- 底部统计和操作 -->
    <div class="footer">
      <div class="stats">
        <el-tooltip content="总窗口数" placement="top">
          <span class="stat-item">
            <el-icon>
              <Monitor />
            </el-icon>
            {{ windowList.length }}
          </span>
        </el-tooltip>

        <el-tooltip content="置顶窗口数" placement="top">
          <span class="stat-item">
            <el-icon><Star /></el-icon>
            {{ topMostCount }}
          </span>
        </el-tooltip>
      </div>

      <div class="footer-actions">
        <el-tooltip content="取消所有置顶" placement="top">
          <el-button
            type="info"
            size="small"
            @click="cancelAllTopMost"
            :disabled="topMostCount === 0"
            class="action-btn"
          >
            <el-icon><Remove /></el-icon>
          </el-button>
        </el-tooltip>
      </div>
    </div>
  </div>
</template>

<style scoped>
.topmost-manager {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #1a2980 0%, #26d0ce 100%);
  border-radius: 0px;
  box-shadow: 0 6px 25px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: "Segoe UI", system-ui, sans-serif;
  -webkit-app-region: no-drag;
}

/* 标题栏样式 */
.header {
  background: rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(12px);
  padding: 8px 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.15);
  -webkit-app-region: drag; /* Tauri 实现拖动的核心 */
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header h3 {
  margin: 0;
  color: white;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.3px;
}

.header-actions {
  display: flex;
}

.header-actions :deep(.el-button) {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  transition: all 0.2s ease;
  margin-left: 4px;
}

.header-actions :deep(.el-button:hover) {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

/* 窗口列表样式 */
.window-list {
  flex: 1;
  padding: 8px;
  /* min-height: 0;
  overflow-y: auto; */
}

.window-item {
  background: rgba(255, 255, 255, 0.95);
  border-radius: 8px;
  margin-bottom: 6px;
  padding: 8px 10px;
  display: flex;
  align-items: center;
  gap: 12px;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 1.5px solid rgba(64, 158, 255, 0.15);
  cursor: pointer;
}

.window-item:hover {
  background: rgba(255, 255, 255, 1);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  border-color: rgba(64, 158, 255, 0.3);
}

.window-item.is-topmost {
  background: linear-gradient(135deg, #f0f9ff 0%, #e6f7ff 100%);
  border-color: #409eff;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.2);
}

/* 程序图标样式 */
.window-icon {
  flex-shrink: 0;
}

.icon-wrapper {
  position: relative;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-icon {
  width: 24px;
  height: 24px;
  object-fit: contain;
  border-radius: 4px;
}

.default-icon {
  color: #409eff;
  font-size: 20px;
}

.active-indicator {
  position: absolute;
  bottom: -2px;
  right: -2px;
  width: 8px;
  height: 8px;
  background: #67c23a;
  border-radius: 50%;
  border: 2px solid white;
}

/* 窗口信息样式 */
.window-info {
  flex: 1;
  min-width: 0;
}

.app-name-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 3px;
}

.app-name {
  font-size: 12px;
  font-weight: 600;
  color: #1a1a1a;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-badges {
  display: flex;
  gap: 4px;
}

:deep(.topmost-badge) {
  font-size: 10px;
  padding: 0 4px;
  height: 18px;
  line-height: 18px;
}

:deep(.active-badge) {
  font-size: 10px;
  padding: 0 4px;
  height: 18px;
  line-height: 18px;
}

.window-title {
  margin-bottom: 4px;
}

.title-text {
  font-size: 11px;
  color: #333;
  opacity: 0.9;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  line-height: 1.4;
}

.window-meta {
  display: flex;
  gap: 10px;
  font-size: 10px;
  color: #666;
}

.window-meta span {
  display: flex;
  align-items: center;
  gap: 3px;
  background: rgba(64, 158, 255, 0.08);
  padding: 1px 5px;
  border-radius: 3px;
}

.window-meta .el-icon {
  font-size: 9px;
}

/* 操作按钮样式 */
.window-actions {
  flex-shrink: 0;
  display: flex;
  gap: 4px;
  opacity: 0.8;
  transition: opacity 0.2s;
}

.window-item:hover .window-actions {
  opacity: 1;
}

.window-actions :deep(.el-button) {
  border: none;
  background: rgba(64, 158, 255, 0.12);
  transition: all 0.2s ease;
}

.window-actions :deep(.el-button:hover) {
  background: rgba(64, 158, 255, 0.25);
  transform: scale(1.1);
}

/* 底部样式 */
.footer {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(12px);
  padding: 4px 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid rgba(255, 255, 255, 0.15);
}

.stats {
  display: flex;
  gap: 15px;
}

.stat-item {
  font-size: 11px;
  color: white;
  display: flex;
  align-items: center;
  gap: 5px;
  opacity: 0.9;
  cursor: default;
}

.stat-item .el-icon {
  font-size: 12px;
  opacity: 0.8;
}

.footer-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  background: rgba(255, 255, 255, 0.25);
  border: none;
  color: white;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.35);
}

.action-btn:disabled {
  /* opacity: 0.5; */
  cursor: not-allowed;
}

/* 滚动条样式 */
:deep(.el-scrollbar__bar.is-vertical) {
  width: 5px;
}

:deep(.el-scrollbar__thumb) {
  background-color: rgba(255, 255, 255, 1);
  /* background-color: red; */
  border-radius: 3px;
}

:deep(.el-scrollbar__thumb:hover) {
  background-color: rgba(255, 255, 255, 0.5);
}
</style>

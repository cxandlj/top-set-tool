<template>
  <div class="simple-settings">
    <!-- 标题 -->
    <div class="simple-header">
      <div class="header-left">
        <el-icon><Setting /></el-icon>
        <span>设置</span>
      </div>
      <div class="header-actions">
        <el-button
          type="primary"
          :icon="Close"
          circle
          size="small"
          @click="closeWindow"
        />
      </div>
    </div>

    <!-- 设置项列表 -->
    <div class="simple-settings-list">
      <!-- 开机启动 -->
      <div class="simple-item">
        <div class="simple-item-label">
          <el-icon><Sunrise /></el-icon>
          <span>开机启动</span>
        </div>
        <el-switch v-model="settings.auto_start" active-color="#409EFF" />
      </div>

      <!-- 关闭行为 -->
      <div class="simple-item">
        <div class="simple-item-label">
          <el-icon><CloseBold /></el-icon>
          <span>关闭窗口时</span>
        </div>
        <div class="simple-radio-group">
          <el-radio-group v-model="settings.app_exit_type">
            <el-radio value="Minimize">最小化</el-radio>
            <el-radio value="Exit">退出</el-radio>
          </el-radio-group>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="simple-actions">
      <el-button type="primary" size="small" @click="saveSettings">
        保存
      </el-button>
      <el-button size="small" @click="resetSettings"> 重置 </el-button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from "vue";
import { Setting, Sunrise, CloseBold, Close } from "@element-plus/icons-vue";
import { APP_SETTING_WINDOW_LABEL } from "@/common/constant";
import { invoke } from "@tauri-apps/api/core";

const settings = ref({
  auto_start: false,
  app_exit_type: "Minimize",
});

const loadSavedSettings = () => {
  invoke("get_app_setting").then((data) => {
    console.log("get_app_setting", data);
    settings.value = data;
  });
};

const saveSettings = () => {
  console.log(11, settings.value);
  invoke("save_app_setting", { settings: settings.value })
    .then(() => {})
    .catch((err) => {
      console.log("save app setting error", err);
    });
};

const resetSettings = () => {
  settings.value.auto_start = false;
  settings.value.app_exit_type = "Minimize";
};

// 关闭窗口
const closeWindow = () => {
  invoke("destroy_window", { windowLabel: APP_SETTING_WINDOW_LABEL }).catch(
    (err) => {
      console.log("close window error", err);
    }
  );
};

onMounted(() => {
  loadSavedSettings();
});
</script>

<style scoped>
.simple-settings {
  width: 300px;
  height: 300px;
  background: white;
  /* border-radius: 8px; */
  /* box-shadow: 0 4px 20px rgba(64, 158, 255, 0.2); */
  /* border: 1px solid #e4e7ed; */
  display: flex;
  flex-direction: column;
  overflow: hidden;
  -webkit-app-region: no-drag;
}

.simple-header {
  background: #409eff;
  color: white;
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  font-size: 14px;
  -webkit-app-region: drag; /* Tauri 实现拖动的核心 */
}
.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
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

.simple-settings-list {
  flex: 1;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.simple-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.simple-item:hover {
  background-color: #f5f7fa;
}

.simple-item-label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #606266;
  font-size: 13px;
}

.simple-item-label .el-icon {
  color: #409eff;
}

.simple-radio-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.simple-actions {
  padding: 12px 16px;
  border-top: 1px solid #e4e7ed;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  background: #f8fafc;
}
</style>

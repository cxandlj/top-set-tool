import { ElMessage, ElMessageBox } from "element-plus";
export function elmessage_success(message, showClose, duration) {
  ElMessage({
    showClose,
    message,
    type: "success",
    duration,
  });
}

export function elmessage_error(message, showClose, duration) {
  console.log(1111);
  ElMessage({
    showClose,
    message,
    type: "error",
    duration,
  });
}

export function elmessage_info(message, showClose, duration) {
  ElMessage({
    showClose,
    message,
    type: "info",
    duration,
  });
}

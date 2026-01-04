import { createWebHashHistory, createRouter } from "vue-router";

const routes = [
  { path: "/", component: () => import("./pages/index.vue") },
  { path: "/setting", component: () => import("./pages/setting.vue") },
];

export default createRouter({
  history: createWebHashHistory(),
  routes,
});

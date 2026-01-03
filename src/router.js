import { createWebHashHistory, createRouter } from "vue-router";

const routes = [{ path: "/", component: import("./App.vue") }];

export default createRouter({
  history: createWebHashHistory(),
  routes,
});

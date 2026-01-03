// import JsonView from './pages/json.vue'
// import RegexView from './pages/regex.vue'
// import TestView from './pages/test.vue'
// import HomeView from './pages/home.vue'
// import Test2View from './pages/test2.vue'
// import SysJobView from './pages/sys/job.vue'
// import SysJobEditView from './pages/sys/jobEdit.vue'

const EncryptView = () => import("./pages/common/encrypt.vue");
const JsonView = () => import("./pages/common/json.vue");
const RegexView = () => import("./pages/common/regex.vue");
const TestView = () => import("./pages/test/test.vue");
const HomeView = async () => import("./pages/home.vue");
const Test2View = () => import("./pages/test/test2.vue");
const SysJobView = async () => import("./pages/sys/job.vue");
const SysJobEditView = async () => import("./pages/sys/jobEdit.vue");
const DayJobView = async () => import("./pages/personal/dayJob.vue");
const SysWindowView = async () => import("./pages/sys/window.vue");

export default {
  EncryptView,
  JsonView,
  RegexView,
  TestView,
  Test2View,
  HomeView,
  SysJobView,
  SysJobEditView,
  DayJobView,
  SysWindowView,
};

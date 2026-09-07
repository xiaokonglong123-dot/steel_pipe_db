import { createApp } from "vue"
import { VueQueryPlugin, QueryClient } from "@tanstack/vue-query"
import "element-plus/dist/index.css"
import "./styles/main.css"
import App from "./App.vue"
import { router } from "./router"
import { pinia } from "./stores/pinia"

const app = createApp(App)
app.use(pinia).use(router).use(VueQueryPlugin, { queryClient: new QueryClient() }).mount("#app")

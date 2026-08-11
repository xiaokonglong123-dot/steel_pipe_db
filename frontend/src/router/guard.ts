import type { Router } from "vue-router"
import { useAuthStore } from "@/stores/auth"

export function installGuard(router: Router): void {
  router.beforeEach((to) => {
    const auth = useAuthStore()
    if (to.path !== "/login" && !auth.auth_token) return "/login"
    const permission = to.meta.permission
    if (typeof permission === "string" && !auth.permissions.includes(permission) && !auth.permissions.includes("*") && auth.auth_user?.username !== "admin") return "/"
    if (to.path === "/login" && auth.auth_token) return "/"
    return true
  })
}

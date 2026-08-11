import { computed } from "vue"
import { useAuthStore } from "@/stores/auth"

export function useHasPermission() {
  const auth = useAuthStore()
  return computed(() => (permission: string) => auth.permissions.includes(permission) || auth.permissions.includes("*") || auth.auth_user?.username === "admin")
}

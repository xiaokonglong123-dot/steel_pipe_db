import { computed, ref } from "vue"
import { defineStore } from "pinia"
import { get, post } from "@/api/client"
import type { User } from "@/types"

const tokenKey = "auth_token"
const userKey = "auth_user"

function readUser(): User | null {
  const raw = localStorage.getItem(userKey)
  if (!raw) return null
  try {
    return JSON.parse(raw) as User
  } catch (error) {
    if (error instanceof SyntaxError) return null
    throw error
  }
}

export const useAuthStore = defineStore("auth", () => {
  const auth_token = ref(localStorage.getItem(tokenKey) ?? "")
  const auth_user = ref<User | null>(readUser())
  const permissions = computed(() => auth_user.value?.permissions ?? [])
  function setAuth(token: string, user: User): void {
    auth_token.value = token
    auth_user.value = user
    localStorage.setItem(tokenKey, token)
    localStorage.setItem(userKey, JSON.stringify(user))
  }
  function clearAuth(): void {
    auth_token.value = ""
    auth_user.value = null
    localStorage.removeItem(tokenKey)
    localStorage.removeItem(userKey)
  }
  async function loadMe(): Promise<void> {
    if (auth_token.value) auth_user.value = await get<User>("/auth/me")
  }
  async function logout(): Promise<void> {
    try { await post<void>("/auth/logout") } finally { clearAuth() }
  }
  return { auth_token, auth_user, permissions, setAuth, clearAuth, loadMe, logout }
})

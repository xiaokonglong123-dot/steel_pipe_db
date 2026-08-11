<script setup lang="ts">
import { reactive, ref } from "vue"
import { useRouter } from "vue-router"
import { ElMessage } from "element-plus"
import { post } from "@/api/client"
import { useAuthStore } from "@/stores/auth"
import type { User } from "@/types"
const router = useRouter(); const auth = useAuthStore(); const loading = ref(false)
const form = reactive({ username: "admin", password: "admin123" })
async function login(): Promise<void> { loading.value = true; try { const data = await post<{ readonly user: User; readonly access_token: string }>("/auth/login", form); auth.setAuth(data.access_token, data.user); await router.push("/") } catch (error) { ElMessage.error(error instanceof Error ? error.message : "登录失败") } finally { loading.value = false } }
</script>
<template><main class="login-page"><el-card class="login-card"><h1>ERP 管理系统</h1><p>企业资源计划工作台</p><el-form :model="form" @submit.prevent="login"><el-form-item label="用户名"><el-input v-model="form.username" autocomplete="username" /></el-form-item><el-form-item label="密码"><el-input v-model="form.password" type="password" show-password autocomplete="current-password" /></el-form-item><el-button type="primary" native-type="submit" :loading="loading" class="login-button">登录</el-button></el-form></el-card></main></template>
<style scoped>.login-page { min-height: 100vh; display: grid; place-items: center; background: var(--surface); }.login-card { width: min(420px, calc(100% - 40px)); }.login-card h1 { margin: 0; }.login-card p { color: var(--muted); }.login-button { width: 100%; }</style>

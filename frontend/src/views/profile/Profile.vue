<script setup lang="ts">
import { useAuthStore } from "@/stores/auth"
const auth = useAuthStore()
</script>
<template><el-card><template #header>个人资料</template><el-descriptions :column="1" border><el-descriptions-item label="用户 ID">{{ auth.auth_user?.id }}</el-descriptions-item><el-descriptions-item label="用户名">{{ auth.auth_user?.username }}</el-descriptions-item><el-descriptions-item label="显示名称">{{ auth.auth_user?.display_name }}</el-descriptions-item><el-descriptions-item label="权限"><el-tag v-for="permission in auth.permissions" :key="permission" class="permission">{{ permission }}</el-tag></el-descriptions-item></el-descriptions></el-card></template>
<style scoped>.permission { margin-right: 8px; }</style>

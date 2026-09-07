<script setup lang="ts">
import { onMounted, reactive, ref } from "vue"
import { ElMessage, ElMessageBox } from "element-plus"
import { DataTable, PageHeader } from "@/components"
import { listUsers, createUser, updateUser, deleteUser, listRoles } from "@/api/auth"
import type { Role, User } from "@/types/auth"

const rows = ref<User[]>([])
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)
const loading = ref(false)

const roles = ref<Role[]>([])

const dialog = ref(false)
const editingId = ref<number | null>(null)
const saving = ref(false)
const form = reactive<{
  username: string; password: string; display_name: string; email: string; phone: string;
  is_active: boolean; role_ids: number[];
}>({
  username: "", password: "", display_name: "", email: "", phone: "", is_active: true, role_ids: [],
})

async function load(): Promise<void> {
  loading.value = true
  try {
    const r = await listUsers(page.value, pageSize.value)
    rows.value = [...r.items]
    total.value = r.total
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "加载失败")
  } finally {
    loading.value = false
  }
}

async function loadRoles(): Promise<void> {
  try {
    roles.value = [...(await listRoles())]
  } catch {
    ElMessage.warning("加载角色失败")
  }
}

function openCreate(): void {
  editingId.value = null
  form.username = ""
  form.password = ""
  form.display_name = ""
  form.email = ""
  form.phone = ""
  form.is_active = true
  form.role_ids = []
  dialog.value = true
}

function openEdit(row: User): void {
  editingId.value = row.id
  form.username = row.username
  form.password = ""
  form.display_name = row.display_name
  form.email = row.email ?? ""
  form.phone = row.phone ?? ""
  form.is_active = row.is_active
  form.role_ids = []
  dialog.value = true
}

async function save(): Promise<void> {
  saving.value = true
  try {
    if (editingId.value === null) {
      if (!form.username.trim() || !form.password) {
        ElMessage.warning("请填写用户名和密码")
        return
      }
      await createUser({
        username: form.username,
        password: form.password,
        display_name: form.display_name || form.username,
        role_ids: form.role_ids,
      })
      ElMessage.success("用户已创建")
    } else {
      await updateUser(editingId.value, {
        display_name: form.display_name,
        ...(form.email ? { email: form.email } : {}),
        ...(form.phone ? { phone: form.phone } : {}),
        is_active: form.is_active,
        ...(form.password ? { password: form.password } : {}),
      })
      ElMessage.success("用户已更新")
    }
    dialog.value = false
    void load()
  } catch (error) {
    ElMessage.error(error instanceof Error ? error.message : "保存失败")
  } finally {
    saving.value = false
  }
}

async function remove(row: User): Promise<void> {
  await ElMessageBox.confirm(`确定停用用户「${row.username}」吗？`, "提示", { type: "warning" })
  await deleteUser(row.id)
  ElMessage.success("已停用")
  void load()
}

onMounted(() => {
  void load()
  void loadRoles()
})
</script>

<template>
  <section class="page">
    <PageHeader title="用户管理" subtitle="管理员维护用户与角色（仅 admin）">
      <el-button type="primary" @click="openCreate">新建用户</el-button>
    </PageHeader>

    <el-card>
      <DataTable
        :columns="[
          { prop: 'username', label: '用户名' },
          { prop: 'display_name', label: '显示名' },
          { prop: 'email', label: '邮箱' },
          { prop: 'phone', label: '电话' },
          { prop: 'created_at', label: '创建时间' },
        ]"
        :data="rows"
        :total="total"
        :page="page"
        :page-size="pageSize"
        :loading="loading"
        @page-change="page = $event; load()"
        @page-size-change="pageSize = $event; page = 1; load()"
      >
        <template #cell-username="{ row }">
          {{ (row as User).username }}
          <el-tag v-if="!(row as User).is_active" type="info" size="small">停用</el-tag>
        </template>
        <template #actions="{ row }">
          <el-button link type="primary" @click="openEdit(row as User)">编辑</el-button>
          <el-button link type="danger" @click="remove(row as User)">停用</el-button>
        </template>
      </DataTable>
    </el-card>

    <el-dialog v-model="dialog" :title="editingId === null ? '新建用户' : '编辑用户'" width="520px" destroy-on-close>
      <el-form label-width="100px">
        <el-form-item label="用户名" required>
          <el-input v-model="form.username" :disabled="editingId !== null" />
        </el-form-item>
        <el-form-item :label="editingId === null ? '密码' : '重置密码'">
          <el-input v-model="form.password" type="password" show-password :placeholder="editingId !== null ? '留空则不修改' : ''" />
        </el-form-item>
        <el-form-item label="显示名"><el-input v-model="form.display_name" /></el-form-item>
        <el-form-item label="邮箱"><el-input v-model="form.email" /></el-form-item>
        <el-form-item label="电话"><el-input v-model="form.phone" /></el-form-item>
        <el-form-item v-if="editingId === null" label="角色">
          <el-select v-model="form.role_ids" multiple style="width: 100%">
            <el-option v-for="r in roles" :key="r.id" :value="r.id" :label="r.name" />
          </el-select>
        </el-form-item>
        <el-form-item v-else label="启用">
          <el-switch v-model="form.is_active" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialog = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </section>
</template>
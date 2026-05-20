# CONTRIBUTING

## 开发流程

1. Fork 本仓库
2. 从 `main` 创建功能分支: `git checkout -b feat/my-feature`
3. 提交变更确保编译通过
4. 推送分支并创建 Pull Request

## 提交规范

使用 Conventional Commits 格式:

```
feat: 新功能
fix: 修复 bug
refactor: 重构（不涉及功能变更）
docs: 文档变更
chore: 工具/配置变更
test: 测试相关
```

## 本地开发检查清单

提交前请确认：

- [ ] `cargo check` 通过（backend/）
- [ ] `npx tsc --noEmit` 通过（frontend/）
- [ ] 未引入新的 lint 警告
- [ ] `.env` 中的敏感信息未提交

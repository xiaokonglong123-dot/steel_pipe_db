# 前端 — React 19 包

## 技术栈
- **React 19** — UI 库
- **Vite** — 构建工具（vanilla-ts 模板）
- **TypeScript** — 严格模式
- **Ant Design 5** — UI 组件
- **TanStack Query (React Query 5)** — 服务端状态
- **react-router-dom v7** — 路由
- **axios** — HTTP 客户端
- **i18next / react-i18next** — 国际化（主要 zh-CN，后备 en-US）
- **dayjs** — 日期处理
- **zod** — 模式验证

## 构建与开发
```bash
cd frontend
npm install        # 安装依赖（包括 vite、antd 等）
npm run dev        # 开发服务器，位于 http://localhost:5173
npm run build      # 生产构建到 dist/
npm run lint       # ESLint
npm run preview    # 预览生产构建
```

## 包架构

```
frontend/
├── public/              ← 静态资源
├── src/
│   ├── main.tsx         ← React 入口、i18n 初始化、QueryClient 设置
│   ├── App.tsx          ← 路由设置、布局、认证守卫
│   ├── api/             ← 共享：axios 实例、拦截器
│   ├── components/      ← 共享：布局、通用组件
│   ├── hooks/           ← 共享：自定义 React hooks
│   ├── lib/             ← validateResponse.ts，运行时 zod 响应验证
│   ├── i18n/            ← 翻译资源 (zh, en)
│   ├── routes/          ← 路由定义 (react-router)
│   ├── theme/           ← Ant Design 主题配置
│   ├── zod-schemas/     ← 7 个 Zod 模式文件，用于 API 响应验证
│   ├── utils/           ← 工具函数
│   └── features/        ← 特性模块（详见 features/AGENTS_zh.md）
│       ├── auth/
│       ├── pipes/
│       ├── inventory/
│       ├── suppliers/
│       ├── customers/
│       ├── purchases/
│       ├── sales/
│       ├── quality/
│       ├── contracts/
│       ├── reports/
│       └── labels/
├── index.html
├── vite.config.ts       ← React 插件、代理、manualChunks 供应商分包
├── tsconfig.json        ← 严格 TypeScript 配置
├── .eslintrc.cjs        ← ESLint 配置
├── .prettierrc          ← Prettier 配置 (singleQuote, 2 space, noBracketSpacing)
└── package.json
```

## 关键依赖（来自 package.json）
- `react`、`react-dom` (^19)
- `antd` (^5) — UI 库
- `@tanstack/react-query` (^5) — 服务端状态
- `react-router-dom` (^7) — 客户端路由
- `axios` (^1) — HTTP 客户端
- `i18next`、`react-i18next` — 国际化
- `dayjs` — 日期工具
- `zod` — 模式验证

## 约定
- 基于特性组织的目录结构，位于 `src/features/`
- 所有 API 调用通过 `@tanstack/react-query` hooks（组件中不直接 fetch）
- i18n 命名空间按特性划分：`pipes:`、`inventory:` 等
- Ant Design 组件 + 主题配置位于 `src/theme/`
- Vite 开发代理：`/api/*` → `http://localhost:3000`
- TypeScript 严格模式已启用
- 禁止使用 `as any`、`@ts-ignore`、`@ts-expect-error`
- vite.config.ts 中的供应商分包策略：antd→vendor-antd、react 生态→vendor-react、工具库→vendor-utils、应用代码→index（约 162 kB gzip）

## 关键文件
- `vite.config.ts` — Vite 配置（React 插件、代理、manualChunks 供应商分包）
- `tsconfig.json` — TypeScript 配置（严格、JSX react-jsx）
- `.eslintrc.cjs` — ESLint 规则
- `.prettierrc` — `singleQuote: true, tabWidth: 2, bracketSpacing: false`

// i18n 初始化 — 动态加载功能模块翻译
// 仅 common.json 静态导入（登录页 + 主布局必需）
// 其他 20 个功能模块通过 import() 动态加载（Vite 自动拆分为独立 chunk）

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhCommon from './locales/zh/common.json';
import enCommon from './locales/en/common.json';

const FEATURE_FILES = [
  'pipes',
  'screen_pipes',
  'inventory',
  'inbound',
  'outbound',
  'stock',
  'location',
  'inventory_check',
  'purchase',
  'sales',
  'quality',
  'contracts',
  'suppliers',
  'customers',
  'reports',
  'labels',
  'profile',
  'search',
  'system',
  'validation',
] as const;

type FeatureKey = (typeof FEATURE_FILES)[number];

// 异步加载某个语言的翻译 JSON
async function importFeature(
  lang: 'zh' | 'en',
  name: string,
): Promise<Record<string, unknown>> {
  const mod = await import(`./locales/${lang}/${name}.json`);
  return (mod.default ?? mod) as Record<string, unknown>;
}

function buildLoaders(
  lang: 'zh' | 'en',
): Record<string, () => Promise<Record<string, unknown>>> {
  return Object.fromEntries(
    FEATURE_FILES.map((f) => [f, () => importFeature(lang, f)]),
  );
}

const ZH_LOADERS = buildLoaders('zh');
const EN_LOADERS = buildLoaders('en');

function getLoaders(lang: string) {
  return lang === 'en' ? EN_LOADERS : ZH_LOADERS;
}

i18n
  .use(initReactI18next)
  .init({
    resources: {
      zh: { translation: { ...zhCommon } as Record<string, unknown> },
      en: { translation: { ...enCommon } as Record<string, unknown> },
    },
    lng: localStorage.getItem('locale') || 'zh',
    fallbackLng: 'zh',
    interpolation: {
      escapeValue: false,
    },
  });

/**
 * 按需加载某个功能模块的翻译。
 *
 * 通过 addResourceBundle 将翻译键嵌套到 translation namespace 下，
 * 保持与之前静态导入相同的数据结构。
 *
 * 例如 loadFeatureTranslations('pipes') 后，组件中可使用 t('pipes.pipe_number')
 */
export async function loadFeatureTranslations(
  key: FeatureKey | string,
): Promise<void> {
  const lang = i18n.language;
  const loaders = getLoaders(lang);
  const loader = loaders[key];
  if (!loader) return;

  try {
    const data = await loader();
    if (key === 'purchase') {
      // purchase.json 同时注册到 'purchases' 和 'purchase' 两个 key 下
      i18n.addResourceBundle(lang, 'translation', { purchases: data, purchase: data }, true, true);
    } else {
      i18n.addResourceBundle(lang, 'translation', { [key]: data }, true, true);
    }
  } catch (err) {
    console.error(`[i18n] Failed to load ${lang}/${key}.json:`, err);
  }
}

// 启动后立即在后台加载所有功能模块翻译（不阻塞首屏渲染）
(async () => {
  const lang = i18n.language;
  const loaders = getLoaders(lang);
  await Promise.allSettled(
    Object.keys(loaders).map((key) => loadFeatureTranslations(key)),
  );
})();

// 语言切换时自动加载对应语言的翻译
i18n.on('languageChanged', (lng: string) => {
  const loaders = getLoaders(lng);
  Object.keys(loaders).forEach((key) => loadFeatureTranslations(key));
});

export default i18n;

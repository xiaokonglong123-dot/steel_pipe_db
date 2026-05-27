// i18n init — lazy-load feature translations
// Only common.json is imported statically (needed for login page + main layout)
// Other 20 feature modules loaded dynamically via import() (Vite auto-splits into separate chunks)

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhCommon from './locales/zh/common.json';
import enCommon from './locales/en/common.json';

type TranslationLoader = () => Promise<unknown>;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function normalizeModule(mod: unknown): Record<string, unknown> {
  if (isRecord(mod) && isRecord(mod.default)) {
    return mod.default;
  }
  if (isRecord(mod)) {
    return mod;
  }
  return {};
}

const ZH_LOADERS = {
  pipes: () => import('./locales/zh/pipes.json'),
  screen_pipes: () => import('./locales/zh/screen_pipes.json'),
  inventory: () => import('./locales/zh/inventory.json'),
  inbound: () => import('./locales/zh/inbound.json'),
  outbound: () => import('./locales/zh/outbound.json'),
  stock: () => import('./locales/zh/stock.json'),
  location: () => import('./locales/zh/location.json'),
  inventory_check: () => import('./locales/zh/inventory_check.json'),
  purchase: () => import('./locales/zh/purchase.json'),
  sales: () => import('./locales/zh/sales.json'),
  quality: () => import('./locales/zh/quality.json'),
  contracts: () => import('./locales/zh/contracts.json'),
  suppliers: () => import('./locales/zh/suppliers.json'),
  customers: () => import('./locales/zh/customers.json'),
  reports: () => import('./locales/zh/reports.json'),
  labels: () => import('./locales/zh/labels.json'),
  profile: () => import('./locales/zh/profile.json'),
  search: () => import('./locales/zh/search.json'),
  system: () => import('./locales/zh/system.json'),
  validation: () => import('./locales/zh/validation.json'),
} satisfies Record<string, TranslationLoader>;

const EN_LOADERS = {
  pipes: () => import('./locales/en/pipes.json'),
  screen_pipes: () => import('./locales/en/screen_pipes.json'),
  inventory: () => import('./locales/en/inventory.json'),
  inbound: () => import('./locales/en/inbound.json'),
  outbound: () => import('./locales/en/outbound.json'),
  stock: () => import('./locales/en/stock.json'),
  location: () => import('./locales/en/location.json'),
  inventory_check: () => import('./locales/en/inventory_check.json'),
  purchase: () => import('./locales/en/purchase.json'),
  sales: () => import('./locales/en/sales.json'),
  quality: () => import('./locales/en/quality.json'),
  contracts: () => import('./locales/en/contracts.json'),
  suppliers: () => import('./locales/en/suppliers.json'),
  customers: () => import('./locales/en/customers.json'),
  reports: () => import('./locales/en/reports.json'),
  labels: () => import('./locales/en/labels.json'),
  profile: () => import('./locales/en/profile.json'),
  search: () => import('./locales/en/search.json'),
  system: () => import('./locales/en/system.json'),
  validation: () => import('./locales/en/validation.json'),
} satisfies Record<keyof typeof ZH_LOADERS, TranslationLoader>;

type FeatureKey = keyof typeof ZH_LOADERS;

function getLoaders(lang: string): Record<string, TranslationLoader> {
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
 * Lazy-load translations for a specific feature module.
 *
 * Uses addResourceBundle to nest translation keys under the translation namespace,
 * keeping the same data structure as the earlier static imports.
 *
 * e.g. after loadFeatureTranslations('pipes'), components can use t('pipes.pipe_number')
 */
export async function loadFeatureTranslations(
  key: FeatureKey | string,
): Promise<void> {
  const lang = i18n.language;
  const loaders = getLoaders(lang);
  const loader = loaders[key];
  if (!loader) return;

  try {
    const data = normalizeModule(await loader());
    if (key === 'purchase') {
      // purchase.json registers under both 'purchases' and 'purchase' keys
      i18n.addResourceBundle(lang, 'translation', { purchases: data, purchase: data }, true, true);
    } else {
      i18n.addResourceBundle(lang, 'translation', { [key]: data }, true, true);
    }
  } catch (err) {
    console.error(`[i18n] Failed to load ${lang}/${key}.json:`, err);
  }
}

// Kick off background loading of all feature translations right after init (doesn't block first paint)
(async () => {
  const lang = i18n.language;
  const loaders = getLoaders(lang);
  await Promise.allSettled(
    Object.keys(loaders).map((key) => loadFeatureTranslations(key)),
  );
})();

// Auto-load translations for the new language on switch
i18n.on('languageChanged', (lng: string) => {
  const loaders = getLoaders(lng);
  Object.keys(loaders).forEach((key) => loadFeatureTranslations(key));
});

export default i18n;

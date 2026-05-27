// i18n init — lazy-load feature translations
// Only common.json is imported statically (needed for login page + main layout)
// Other 20 feature modules loaded dynamically via import() (Vite auto-splits into separate chunks)

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

// Async-load translation JSON for a given language
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
    const data = await loader();
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

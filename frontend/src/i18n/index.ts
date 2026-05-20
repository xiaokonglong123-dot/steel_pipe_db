import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import zhCommon from '../i18n/resources/zh/common.json';
import enCommon from '../i18n/resources/en/common.json';

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      zh: { common: zhCommon },
      en: { common: enCommon },
    },
    fallbackLng: 'zh',
    ns: ['common', 'pipes', 'inventory', 'quality', 'orders', 'contracts', 'reports', 'labels', 'system', 'validation'],
    defaultNS: 'common',
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
  });

export default i18n;

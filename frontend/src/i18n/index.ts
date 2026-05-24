// i18n 初始化 — 自动加载所有翻译命名空间
// common.json 使用扁平点号键（如 app.title），feature JSON 文件自动嵌套在对应命名空间下
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhCommon from './locales/zh/common.json';
import enCommon from './locales/en/common.json';
import zhPipes from './locales/zh/pipes.json';
import enPipes from './locales/en/pipes.json';
import zhScreenPipes from './locales/zh/screen_pipes.json';
import enScreenPipes from './locales/en/screen_pipes.json';
import zhInventory from './locales/zh/inventory.json';
import enInventory from './locales/en/inventory.json';
import zhInbound from './locales/zh/inbound.json';
import enInbound from './locales/en/inbound.json';
import zhOutbound from './locales/zh/outbound.json';
import enOutbound from './locales/en/outbound.json';
import zhStock from './locales/zh/stock.json';
import enStock from './locales/en/stock.json';
import zhLocation from './locales/zh/location.json';
import enLocation from './locales/en/location.json';
import zhInventoryCheck from './locales/zh/inventory_check.json';
import enInventoryCheck from './locales/en/inventory_check.json';
import zhPurchase from './locales/zh/purchase.json';
import enPurchase from './locales/en/purchase.json';
import zhSales from './locales/zh/sales.json';
import enSales from './locales/en/sales.json';
import zhQuality from './locales/zh/quality.json';
import enQuality from './locales/en/quality.json';
import zhContracts from './locales/zh/contracts.json';
import enContracts from './locales/en/contracts.json';
import zhSuppliers from './locales/zh/suppliers.json';
import enSuppliers from './locales/en/suppliers.json';
import zhCustomers from './locales/zh/customers.json';
import enCustomers from './locales/en/customers.json';
import zhReports from './locales/zh/reports.json';
import enReports from './locales/en/reports.json';
import zhLabels from './locales/zh/labels.json';
import enLabels from './locales/en/labels.json';
import zhProfile from './locales/zh/profile.json';
import enProfile from './locales/en/profile.json';
import zhSearch from './locales/zh/search.json';
import enSearch from './locales/en/search.json';
import zhSystem from './locales/zh/system.json';
import enSystem from './locales/en/system.json';
import zhValidation from './locales/zh/validation.json';
import enValidation from './locales/en/validation.json';

// 将扁平 JSON 嵌套在指定 key 下，例如 { pipe_number: "管号" } → { pipes: { pipe_number: "管号" } }
function nestUnder(prefix: string, data: Record<string, unknown>): Record<string, unknown> {
  return { [prefix]: data };
}

i18n.use(initReactI18next).init({
  resources: {
    zh: {
      translation: {
        ...zhCommon,
        ...nestUnder('pipes', zhPipes),
        ...nestUnder('screen_pipes', zhScreenPipes),
        ...nestUnder('inventory', zhInventory),
        ...nestUnder('inbound', zhInbound),
        ...nestUnder('outbound', zhOutbound),
        ...nestUnder('stock', zhStock),
        ...nestUnder('location', zhLocation),
        ...nestUnder('inventory_check', zhInventoryCheck),
        ...nestUnder('purchases', zhPurchase),
        ...nestUnder('purchase', zhPurchase),
        ...nestUnder('sales', zhSales),
        ...nestUnder('quality', zhQuality),
        ...nestUnder('contracts', zhContracts),
        ...nestUnder('suppliers', zhSuppliers),
        ...nestUnder('customers', zhCustomers),
        ...nestUnder('reports', zhReports),
        ...nestUnder('labels', zhLabels),
        ...nestUnder('profile', zhProfile),
        ...nestUnder('search', zhSearch),
        ...nestUnder('system', zhSystem),
        ...nestUnder('validation', zhValidation),
      },
    },
    en: {
      translation: {
        ...enCommon,
        ...nestUnder('pipes', enPipes),
        ...nestUnder('screen_pipes', enScreenPipes),
        ...nestUnder('inventory', enInventory),
        ...nestUnder('inbound', enInbound),
        ...nestUnder('outbound', enOutbound),
        ...nestUnder('stock', enStock),
        ...nestUnder('location', enLocation),
        ...nestUnder('inventory_check', enInventoryCheck),
        ...nestUnder('purchases', enPurchase),
        ...nestUnder('purchase', enPurchase),
        ...nestUnder('sales', enSales),
        ...nestUnder('quality', enQuality),
        ...nestUnder('contracts', enContracts),
        ...nestUnder('suppliers', enSuppliers),
        ...nestUnder('customers', enCustomers),
        ...nestUnder('reports', enReports),
        ...nestUnder('labels', enLabels),
        ...nestUnder('profile', enProfile),
        ...nestUnder('search', enSearch),
        ...nestUnder('system', enSystem),
        ...nestUnder('validation', enValidation),
      },
    },
  },
  lng: localStorage.getItem('locale') || 'zh',
  fallbackLng: 'zh',
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;

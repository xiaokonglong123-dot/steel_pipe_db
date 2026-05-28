/**
 * Main layout — sidebar nav + top bar with user info + content area
 *
 * All protected pages render inside this layout's Outlet.
 * The sidebar groups menu items by module (pipes, inventory, suppliers, customers, purchase/sales orders, etc),
 * and the top-right shows the current user with a logout dropdown.
 */
import { Suspense } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { Layout, Menu, Button, Typography, Dropdown, Spin } from 'antd';
import {
  TeamOutlined,
  ShopOutlined,
  ShoppingCartOutlined,
  DollarOutlined,
  SafetyCertificateOutlined,
  FileTextOutlined,
  BarChartOutlined,
  BarcodeOutlined,
  LogoutOutlined,
  UserOutlined,
  ContainerOutlined,
  ImportOutlined,
  SearchOutlined,
  SettingOutlined,
} from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '@/stores/authStore';
import { useAppStore } from '@/stores/appStore';
import type { MenuProps } from 'antd';

const { Header, Sider, Content } = Layout;
const { Text } = Typography;

export default function MainLayout() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const user = useAuthStore((s) => s.user);
  const logout = useAuthStore((s) => s.logout);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  // 侧边栏菜单结构：按业务模块分组
  const menuItems: MenuProps['items'] = [
    {
      key: 'pipes',
      icon: <ContainerOutlined />,
      label: t('menu.pipes'),
      children: [
        { key: '/pipes/seamless', label: t('menu.seamless_pipes') },
        { key: '/pipes/screen', label: t('menu.screen_pipes') },
      ],
    },
    {
      key: 'inventory',
      icon: <ShopOutlined />,
      label: t('menu.inventory'),
      children: [
        { key: '/inventory/inbound', label: t('menu.inbound') },
        { key: '/inventory/outbound', label: t('menu.outbound') },
        { key: '/inventory/stock', label: t('menu.stock_query') },
        { key: '/inventory/locations', label: t('menu.locations') },
        { key: '/inventory/check', label: t('menu.inventory_check') },
      ],
    },
    {
      key: 'suppliers',
      icon: <TeamOutlined />,
      label: t('menu.suppliers'),
      children: [{ key: '/suppliers', label: t('menu.supplier_list') }],
    },
    {
      key: 'customers',
      icon: <TeamOutlined />,
      label: t('menu.customers'),
      children: [{ key: '/customers', label: t('menu.customer_list') }],
    },
    {
      key: 'purchases',
      icon: <ShoppingCartOutlined />,
      label: t('menu.purchases'),
      children: [{ key: '/purchases', label: t('menu.purchase_orders') }],
    },
    {
      key: 'sales',
      icon: <DollarOutlined />,
      label: t('menu.sales'),
      children: [{ key: '/sales', label: t('menu.sales_orders') }],
    },
    {
      key: 'quality',
      icon: <SafetyCertificateOutlined />,
      label: t('menu.quality'),
      children: [{ key: '/quality/certs', label: t('menu.quality_certs') }],
    },
    {
      key: 'contracts',
      icon: <FileTextOutlined />,
      label: t('menu.contracts'),
      children: [{ key: '/contracts', label: t('menu.contract_list') }],
    },
    {
      key: 'reports',
      icon: <BarChartOutlined />,
      label: t('menu.reports'),
      children: [
        { key: '/reports', label: t('menu.report_list') },
        { key: '/reports/dashboard', label: t('menu.dashboard') },
      ],
    },
    {
      key: 'labels',
      icon: <BarcodeOutlined />,
      label: t('menu.labels'),
      children: [{ key: '/labels', label: t('menu.label_print') }],
    },
    {
      key: 'search',
      icon: <SearchOutlined />,
      label: t('menu.search'),
      children: [{ key: '/search', label: t('menu.search_global') }],
    },
    {
      key: 'data-io',
      icon: <ImportOutlined />,
      label: t('menu.data_io'),
      children: [
        { key: '/data-io/import', label: t('menu.data_import') },
        { key: '/data-io/export', label: t('menu.data_export') },
        { key: '/data-io/logs', label: t('menu.operation_log') },
      ],
    },
    {
      key: 'system',
      icon: <SettingOutlined />,
      label: t('menu.system'),
      children: [
        { key: '/system/users', label: t('menu.user_management') },
      ],
    },
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: t('menu.profile'),
      children: [{ key: '/profile/settings', label: t('menu.profile_settings') }],
    },
  ];

  // 根据当前路径高亮对应菜单项
  const selectedKeys = [location.pathname];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider theme="dark" collapsible collapsed={sidebarCollapsed} onCollapse={toggleSidebar}>
        <div style={{ padding: 16, textAlign: 'center' }}>
          <Text strong style={{ color: '#fff', fontSize: 16 }}>
            {t('app.title')}
          </Text>
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={selectedKeys}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
        />
      </Sider>
      <Layout>
        <Header
          style={{
            background: '#fff',
            padding: '0 24px',
            display: 'flex',
            justifyContent: 'flex-end',
            alignItems: 'center',
          }}
        >
          {/* 右上角用户信息 & 登出下拉 */}
          <Dropdown
            menu={{
              items: [
                {
                  key: 'profile',
                  icon: <SettingOutlined />,
                  label: t('menu.profile_settings'),
                  onClick: () => navigate('/profile/settings'),
                },
                { type: 'divider' },
                {
                  key: 'logout',
                  icon: <LogoutOutlined />,
                  label: t('common.logout'),
                  onClick: handleLogout,
                },
              ],
            }}
          >
            <Button type="text" icon={<UserOutlined />}>
              {user?.username ?? '-'}
            </Button>
          </Dropdown>
        </Header>
        <Content style={{ margin: 24 }}>
          <Suspense
            fallback={
              <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%', minHeight: 300 }}>
                <Spin size="large" />
              </div>
            }
          >
            <Outlet />
          </Suspense>
        </Content>
      </Layout>
    </Layout>
  );
}

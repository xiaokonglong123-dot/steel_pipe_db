import React from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { Layout, Menu, Button, Dropdown, Space, Switch } from 'antd';
import {
  MenuFoldOutlined, MenuUnfoldOutlined, LogoutOutlined, UserOutlined,
  SettingOutlined, ToolOutlined, DatabaseOutlined, SafetyCertificateOutlined,
  ShoppingCartOutlined, FileTextOutlined, BarChartOutlined, TagOutlined,
  ImportOutlined, TeamOutlined,
} from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '../stores/authStore';
import { useAppStore } from '../stores/appStore';
import { useUnitStore } from '../stores/unitStore';

const { Header, Sider, Content } = Layout;

const menuItems = [
  { key: '/pipes', icon: <ToolOutlined />, label: 'nav.pipes', roles: ['admin', 'warehouse', 'qc', 'sales'] },
  { key: '/inventory', icon: <DatabaseOutlined />, label: 'nav.inventory', roles: ['admin', 'warehouse'] },
  { key: '/quality', icon: <SafetyCertificateOutlined />, label: 'nav.quality', roles: ['admin', 'qc', 'warehouse'] },
  { key: '/orders', icon: <ShoppingCartOutlined />, label: 'nav.orders', roles: ['admin', 'sales', 'warehouse'] },
  { key: '/data-io', icon: <ImportOutlined />, label: 'nav.importExport', roles: ['admin', 'warehouse'] },
  { key: '/contracts', icon: <FileTextOutlined />, label: 'nav.contracts', roles: ['admin', 'sales'] },
  { key: '/reports', icon: <BarChartOutlined />, label: 'nav.reports', roles: ['admin', 'sales', 'warehouse', 'qc'] },
  { key: '/labels', icon: <TagOutlined />, label: 'nav.labels', roles: ['admin', 'warehouse'] },
  { key: '/system/users', icon: <TeamOutlined />, label: 'nav.system', roles: ['admin'] },
];

export default function MainLayout({ children }: { children: React.ReactNode }) {
  const { t, i18n } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const { user, logout } = useAuthStore();
  const { sidebarCollapsed, toggleSidebar } = useAppStore();
  const { unitSystem, toggleUnitSystem } = useUnitStore();

  const filteredMenuItems = menuItems
    .filter((item) => user && (item.roles.includes(user.role) || user.role === 'admin'))
    .map((item) => ({ ...item, label: t(item.label) }));

  const userMenu = {
    items: [
      { key: 'logout', icon: <LogoutOutlined />, label: t('auth.logout') },
    ],
    onClick: ({ key }: { key: string }) => {
      if (key === 'logout') {
        logout();
        navigate('/login');
      }
    },
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider trigger={null} collapsible collapsed={sidebarCollapsed} theme="dark"
        style={{ background: '#0F1A2E' }}>
        <div style={{ height: 64, display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#fff', fontSize: sidebarCollapsed ? 14 : 18, fontWeight: 'bold', padding: '0 16px' }}>
          {sidebarCollapsed ? 'PM' : t('app.shortTitle')}
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[location.pathname.startsWith('/pipes') ? '/pipes' : `/${location.pathname.split('/')[1]}`]}
          items={filteredMenuItems}
          onClick={({ key }) => navigate(key)}
          style={{ background: '#0F1A2E' }}
        />
      </Sider>
      <Layout>
        <Header style={{ padding: '0 24px', background: '#fff', display: 'flex', alignItems: 'center', justifyContent: 'space-between', boxShadow: '0 1px 4px rgba(0,0,0,0.08)' }}>
          <Button type="text" icon={sidebarCollapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />} onClick={toggleSidebar} />
          <Space size="middle">
            <Space>
              <span style={{ fontSize: 12, color: '#999' }}>{t('system.units')}:</span>
              <Switch
                checkedChildren={t('system.units.imperial')}
                unCheckedChildren={t('system.units.metric')}
                checked={unitSystem === 'imperial'}
                onChange={toggleUnitSystem}
                size="small"
              />
            </Space>
            <Dropdown menu={userMenu}>
              <Space style={{ cursor: 'pointer' }}>
                <UserOutlined />
                {user?.display_name}
              </Space>
            </Dropdown>
          </Space>
        </Header>
        <Content style={{ margin: 24, minHeight: 280 }}>
          {children}
        </Content>
      </Layout>
    </Layout>
  );
}

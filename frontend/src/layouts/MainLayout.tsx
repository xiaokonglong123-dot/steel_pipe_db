import { useState } from 'react';
import { Layout, Menu, Button, theme } from 'antd';
import {
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  LogoutOutlined,
} from '@ant-design/icons';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '@/stores/authStore';

const { Header, Sider, Content } = Layout;

export default function MainLayout() {
  const [collapsed, setCollapsed] = useState(false);
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const user = useAuthStore((s) => s.user);
  const clearAuth = useAuthStore((s) => s.clearAuth);
  const {
    token: { colorBgContainer, borderRadiusLG },
  } = theme.useToken();

  const menuItems = [
    {
      key: '/pipes',
      label: t('nav.pipes'),
      children: [
        { key: '/pipes/seamless', label: t('nav.seamless_pipes') },
        { key: '/pipes/screen', label: t('nav.screen_pipes') },
      ],
    },
    {
      key: '/inventory',
      label: t('nav.inventory'),
      children: [
        { key: '/inventory/inbound', label: t('nav.inbound') },
        { key: '/inventory/outbound', label: t('nav.outbound') },
        { key: '/inventory/stock', label: t('nav.stock_query') },
        { key: '/inventory/check', label: t('nav.inventory_check') },
        { key: '/inventory/locations', label: t('nav.locations') },
      ],
    },
    { key: '/quality', label: t('nav.quality') },
    { key: '/purchases', label: t('nav.purchases') },
    { key: '/sales', label: t('nav.sales') },
    { key: '/contracts', label: t('nav.contracts') },
    { key: '/reports', label: t('nav.reports') },
    { key: '/labels', label: t('nav.labels') },
    {
      key: '/system',
      label: t('nav.system'),
      children: [
        { key: '/system/users', label: t('nav.users') },
      ],
    },
  ];

  const handleLogout = () => {
    clearAuth();
    navigate('/login');
  };

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider trigger={null} collapsible collapsed={collapsed}>
        <div
          style={{
            height: 32,
            margin: 16,
            color: '#fff',
            fontWeight: 'bold',
            fontSize: collapsed ? 14 : 18,
            textAlign: 'center',
            whiteSpace: 'nowrap',
            overflow: 'hidden',
          }}
        >
          {collapsed ? 'SPDB' : t('app.title')}
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[location.pathname]}
          defaultOpenKeys={['/pipes', '/inventory']}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
        />
      </Sider>
      <Layout>
        <Header
          style={{
            padding: '0 16px',
            background: colorBgContainer,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <Button
            type="text"
            icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
            onClick={() => setCollapsed(!collapsed)}
          />
          <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
            <span>
              {user?.display_name} ({user?.role})
            </span>
            <Button
              type="text"
              icon={<LogoutOutlined />}
              onClick={handleLogout}
            >
              {t('user.logout')}
            </Button>
          </div>
        </Header>
        <Content
          style={{
            margin: 16,
            padding: 24,
            background: colorBgContainer,
            borderRadius: borderRadiusLG,
            minHeight: 280,
          }}
        >
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
}

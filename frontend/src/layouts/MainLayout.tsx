import { Suspense } from 'react';
import { Outlet, useNavigate, useLocation, useMatches, Navigate } from 'react-router-dom';
import { Layout, Menu, Button, Typography, Dropdown, Spin } from 'antd';
import {
  TeamOutlined,
  AuditOutlined,
  ToolOutlined,
  ProjectOutlined,
  ApartmentOutlined,
  BellOutlined,
  GlobalOutlined,
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
import type { ReactNode } from 'react';

const { Header, Sider, Content } = Layout;
const { Text } = Typography;

interface MenuChild {
  key: string;
  label: ReactNode;
  /** Roles allowed to see this menu entry. Omitted = visible to everyone. */
  roles?: string[];
}

interface MenuGroup {
  key: string;
  icon?: ReactNode;
  label: ReactNode;
  children: MenuChild[];
}

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

  // 侧边栏菜单结构：按业务模块分组（子项可带 roles 做角色过滤）
  const menuItems: MenuGroup[] = [
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
        { key: '/data-io/import', label: t('menu.data_import'), roles: ['admin'] },
        {
          key: '/data-io/export',
          label: t('menu.data_export'),
          roles: ['admin', 'warehouse', 'sales'],
        },
        { key: '/data-io/logs', label: t('menu.operation_log'), roles: ['admin'] },
      ],
    },
    {
      key: 'workflow',
      icon: <AuditOutlined />,
      label: t('menu.workflow'),
      children: [
        { key: '/workflow/my-tasks', label: t('menu.workflow_tasks') },
        { key: '/workflow/definitions', label: t('menu.workflow_definitions'), roles: ['admin'] },
      ],
    },
    {
      key: 'hr',
      icon: <TeamOutlined />,
      label: t('menu.hr'),
      children: [
        { key: '/hr/employees', label: t('menu.hr_employees'), roles: ['admin'] },
        { key: '/hr/salaries', label: t('menu.hr_salaries'), roles: ['admin'] },
      ],
    },
    {
      key: 'finance',
      icon: <DollarOutlined />,
      label: t('menu.finance'),
      children: [{ key: '/finance', label: t('menu.finance_overview'), roles: ['admin'] }],
    },
    {
      key: 'procurement',
      icon: <ShoppingCartOutlined />,
      label: t('menu.procurement'),
      children: [{ key: '/procurement', label: t('menu.procurement_overview'), roles: ['admin'] }],
    },
    {
      key: 'sales-crm',
      icon: <ShopOutlined />,
      label: t('menu.sales_crm'),
      children: [{ key: '/sales/crm', label: t('menu.sales_crm_overview'), roles: ['admin'] }],
    },
    {
      key: 'inventory-atp',
      icon: <ContainerOutlined />,
      label: t('menu.atp'),
      children: [{ key: '/inventory/atp', label: t('menu.atp_overview'), roles: ['admin'] }],
    },
    {
      key: 'manufacturing',
      icon: <ToolOutlined />,
      label: t('menu.manufacturing'),
      children: [
        { key: '/manufacturing', label: t('menu.manufacturing_overview'), roles: ['admin'] },
      ],
    },
    {
      key: 'projects',
      icon: <ProjectOutlined />,
      label: t('menu.projects'),
      children: [{ key: '/projects', label: t('menu.projects_overview'), roles: ['admin'] }],
    },
    {
      key: 'bi',
      icon: <BarChartOutlined />,
      label: t('menu.bi'),
      children: [{ key: '/bi', label: t('menu.bi_dashboard'), roles: ['admin'] }],
    },
    {
      key: 'assets',
      icon: <ApartmentOutlined />,
      label: t('menu.assets'),
      children: [{ key: '/assets', label: t('menu.assets_overview'), roles: ['admin'] }],
    },
    {
      key: 'notifications',
      icon: <BellOutlined />,
      label: t('menu.notifications'),
      children: [{ key: '/notifications', label: t('menu.notifications_inbox') }],
    },
    {
      key: 'portal',
      icon: <GlobalOutlined />,
      label: t('menu.portal'),
      children: [{ key: '/portal', label: t('menu.portal_admin'), roles: ['admin'] }],
    },
    {
      key: 'system',
      icon: <SettingOutlined />,
      label: t('menu.system'),
      children: [
        { key: '/system/users', label: t('menu.user_management'), roles: ['admin'] },
        { key: '/system/roles', label: t('menu.role_management'), roles: ['admin'] },
        { key: '/system/departments', label: t('menu.department_management'), roles: ['admin'] },
      ],
    },
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: t('menu.profile'),
      children: [{ key: '/profile/settings', label: t('menu.profile_settings') }],
    },
  ];

  // 按当前用户角色过滤菜单（后端仍有 RBAC 兜底，前端仅隐藏无权限入口）
  const visibleMenuItems: MenuProps['items'] = menuItems
    .map((group) => {
      const children = group.children.filter(
        (child) => !child.roles || (user ? child.roles.includes(user.role) : false),
      );
      return children.length > 0 ? { ...group, children } : null;
    })
    .filter((group): group is NonNullable<typeof group> => group !== null);

  // 路由级 RBAC：当前匹配路由若声明了 roles，则校验当前用户角色
  const matches = useMatches();
  const denied = matches.some((match) => {
    const roles = (match.handle as { roles?: string[] } | undefined)?.roles;
    return roles ? !user || !roles.includes(user.role) : false;
  });

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
          items={visibleMenuItems}
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
            {denied ? <Navigate to="/" replace /> : <Outlet />}
          </Suspense>
        </Content>
      </Layout>
    </Layout>
  );
}

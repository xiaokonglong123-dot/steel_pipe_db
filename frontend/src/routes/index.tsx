import { createBrowserRouter, Navigate } from 'react-router-dom';
import MainLayout from '@/layouts/MainLayout';
import ProtectedRoute from './ProtectedRoute';
import LoginPage from '@/features/auth/pages/LoginPage';
import SeamlessPipeListPage from '@/features/pipes/pages/SeamlessPipeListPage';
import SeamlessPipeFormPage from '@/features/pipes/pages/SeamlessPipeFormPage';
import SeamlessPipeDetailPage from '@/features/pipes/pages/SeamlessPipeDetailPage';
import ScreenPipeListPage from '@/features/pipes/pages/ScreenPipeListPage';
import ScreenPipeFormPage from '@/features/pipes/pages/ScreenPipeFormPage';
import ScreenPipeDetailPage from '@/features/pipes/pages/ScreenPipeDetailPage';
import InboundListPage from '@/features/inventory/pages/InboundListPage';
import OutboundListPage from '@/features/inventory/pages/OutboundListPage';
import StockQueryPage from '@/features/inventory/pages/StockQueryPage';
import LocationListPage from '@/features/inventory/pages/LocationListPage';
import InventoryCheckListPage from '@/features/inventory/pages/InventoryCheckListPage';

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <MainLayout />
      </ProtectedRoute>
    ),
    children: [
      { index: true, element: <Navigate to="/pipes/seamless" replace /> },
      {
        path: 'pipes/seamless',
        element: <SeamlessPipeListPage />,
      },
      {
        path: 'pipes/seamless/new',
        element: <SeamlessPipeFormPage />,
      },
      {
        path: 'pipes/seamless/:id',
        element: <SeamlessPipeDetailPage />,
      },
      {
        path: 'pipes/seamless/:id/edit',
        element: <SeamlessPipeFormPage />,
      },
      {
        path: 'pipes/screen',
        element: <ScreenPipeListPage />,
      },
      {
        path: 'pipes/screen/new',
        element: <ScreenPipeFormPage />,
      },
      {
        path: 'pipes/screen/:id',
        element: <ScreenPipeDetailPage />,
      },
      {
        path: 'pipes/screen/:id/edit',
        element: <ScreenPipeFormPage />,
      },
      { path: 'inventory/inbound', element: <InboundListPage /> },
      { path: 'inventory/outbound', element: <OutboundListPage /> },
      { path: 'inventory/stock', element: <StockQueryPage /> },
      { path: 'inventory/locations', element: <LocationListPage /> },
      { path: 'inventory/check', element: <InventoryCheckListPage /> },
    ],
  },
]);

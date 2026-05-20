import React from 'react';
import { Routes, Route } from 'react-router-dom';

const UserListPage = React.lazy(() => import('./pages/UserListPage'));

export default function AuthModule() {
  return (
    <Routes>
      <Route path="users" element={<UserListPage />} />
    </Routes>
  );
}

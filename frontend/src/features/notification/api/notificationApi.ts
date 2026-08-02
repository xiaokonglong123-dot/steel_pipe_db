// Notifications API
import apiClient from '@/api/client';
import type { ApiResponse } from '@/types';
import { validateResponse } from '@/lib/validateResponse';
import { z } from 'zod';

export interface Notification {
  id: number;
  title: string;
  content: string | null;
  notify_type: string;
  is_read: boolean;
  created_at: string;
}

const notifSchema = z.object({
  id: z.number(), title: z.string(), content: z.string().nullable(),
  notify_type: z.string(), is_read: z.boolean(), created_at: z.string(),
}).passthrough();
const arrayOf = <T extends z.ZodTypeAny>(item: T) =>
  z.object({ success: z.boolean(), data: z.array(item) }).passthrough();

export const notificationApi = {
  list: async (unreadOnly = false) => {
    const res = await apiClient.get<ApiResponse<Notification[]>>('/notifications', { unread_only: unreadOnly });
    return validateResponse(arrayOf(notifSchema), res.data).data;
  },
  unreadCount: async () => {
    const res = await apiClient.get<ApiResponse<{ unread: number }>>('/notifications/unread-count');
    return res.data.unread;
  },
  markRead: async (id: number) => {
    await apiClient.post(`/notifications/${id}/read`);
  },
};

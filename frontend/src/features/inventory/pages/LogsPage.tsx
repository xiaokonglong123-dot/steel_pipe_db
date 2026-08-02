import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { EmptyState } from '@/shared/components/EmptyState';

export default function InventoryLogsPage() {
  const { t } = useTranslation('inventory');
  return (
    <PageLayout title={t('title')}>
      <EmptyState description={t('log_empty_description')} />
    </PageLayout>
  );
}
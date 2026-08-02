import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { EmptyState } from '@/shared/components/EmptyState';

export default function TrendsPage() {
  const { t } = useTranslation('reports');
  return (
    <PageLayout title={t('menu.trends')}>
      <EmptyState description={t('trends_empty')} />
    </PageLayout>
  );
}
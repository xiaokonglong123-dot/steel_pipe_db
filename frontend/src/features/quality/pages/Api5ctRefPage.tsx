import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { EmptyState } from '@/shared/components/EmptyState';

export default function Api5ctRefPage() {
  const { t } = useTranslation('quality');
  return (
    <PageLayout title={t('title')}>
      <EmptyState description={t('api5ct_empty')} />
    </PageLayout>
  );
}
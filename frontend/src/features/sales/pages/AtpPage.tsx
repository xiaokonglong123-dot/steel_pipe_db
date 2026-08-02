import { useTranslation } from 'react-i18next';
import { PageLayout } from '@/shared/components/PageLayout';
import { EmptyState } from '@/shared/components/EmptyState';

export default function AtpPage() {
  const { t } = useTranslation('atp');
  return (
    <PageLayout title={t('title')}>
      <EmptyState description={t('empty_description')} />
    </PageLayout>
  );
}
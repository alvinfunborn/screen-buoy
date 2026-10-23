import React from 'react';
import { Form, InputNumber } from 'antd';
import { useTranslation } from 'react-i18next';
import { SettingSection } from '../SettingSection';

export const UiAutomationSettings: React.FC = () => {
  const { t } = useTranslation();
  return (
    <SettingSection title={t('settings.groups.detection')} description={t('settings.groups.detectionDescription')}>
      <Form.Item className="setting-row" layout="horizontal"
        label={<span>{t('uiAutomation.collectInterval')}<small>{t('uiAutomation.collectIntervalTooltip')}</small></span>}
        name={['ui_automation', 'collect_interval']}>
        <InputNumber min={100} max={10000} step={50} />
      </Form.Item>
      <Form.Item className="setting-row" layout="horizontal"
        label={<span>{t('uiAutomation.cacheTtl')}<small>{t('uiAutomation.cacheTtlTooltip')}</small></span>}
        name={['ui_automation', 'cache_ttl']}>
        <InputNumber min={100} max={1000000} step={100} />
      </Form.Item>
    </SettingSection>
  );
};

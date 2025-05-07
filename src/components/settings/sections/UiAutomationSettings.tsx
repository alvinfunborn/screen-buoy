import React from 'react';
import { Form, InputNumber, Space, Spin, Typography } from 'antd';
import type { Config } from '../../../types/config';
import '../../../styles/global.css';
import { useTranslation } from 'react-i18next';

const { Title } = Typography;

interface UiAutomationSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const UiAutomationSettings: React.FC<UiAutomationSettingsProps> = ({ onValuesChange }) => {
  const { t } = useTranslation();

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Form.Item
        layout="horizontal"
        label={t('uiAutomation.collectInterval')}
        name={['ui_automation', 'collect_interval']}
        tooltip={t('uiAutomation.collectIntervalTooltip')}
      >
        <InputNumber min={50} max={10000} step={50} />
      </Form.Item>
      <Form.Item
        layout="horizontal"
        label={t('uiAutomation.cacheTtl')}
        name={['ui_automation', 'cache_ttl']}
        tooltip={t('uiAutomation.cacheTtlTooltip')}
      >
        <InputNumber min={100} max={1000000} step={100} />
      </Form.Item>
    </Space>
  );
};

import { useCallback, useEffect, useState } from 'react';
import { Alert, Button, Space, Tag, Typography } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { useTranslation } from 'react-i18next';

interface PermissionStatus {
  required: boolean;
  accessibility: boolean;
  inputMonitoring: boolean;
  keyboardHook: boolean;
  ready: boolean;
  appPath: string;
}

export function MacPermissions() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<PermissionStatus | null>(null);
  const [error, setError] = useState('');
  const [pending, setPending] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await invoke<PermissionStatus>('get_permission_status'));
      setError('');
    } catch (err) {
      setError(String(err));
    }
  }, []);

  useEffect(() => {
    let alive = true;
    const update = async () => {
      try {
        const next = await invoke<PermissionStatus>('get_permission_status');
        if (alive) { setStatus(next); setError(''); }
      } catch (err) {
        if (alive) setError(String(err));
      }
    };
    void update();
    const timer = window.setInterval(() => {
      if (!document.hidden) void update();
    }, 2000);
    window.addEventListener('focus', update);
    return () => {
      alive = false;
      window.clearInterval(timer);
      window.removeEventListener('focus', update);
    };
  }, []);

  const request = async (permission: string) => {
    setPending(permission);
    try {
      await invoke('open_permission_settings', { permission });
      await refresh();
    } catch (err) {
      setError(String(err));
    } finally {
      setPending(null);
    }
  };

  if (!status?.required) {
    return error ? <Alert type="error" message={t('permissions.checkFailed')} description={error} /> : null;
  }

  return (
    <Alert
      style={{ marginBottom: 20 }}
      showIcon
      type={status.ready ? 'success' : 'warning'}
      message={t(status.ready ? 'permissions.ready' : 'permissions.title')}
      description={
        <Space direction="vertical" style={{ width: '100%' }}>
          {!status.ready && <span>{t('permissions.description')}</span>}
          <Space wrap>
            <Tag color={status.accessibility ? 'green' : 'orange'}>
              {t('permissions.accessibility')}: {t(status.accessibility ? 'permissions.granted' : 'permissions.missing')}
            </Tag>
            <Tag color={status.inputMonitoring ? 'green' : 'default'}>
              {t('permissions.inputMonitoring')}: {t(status.inputMonitoring ? 'permissions.granted' : 'permissions.notGranted')}
            </Tag>
            <Tag color={status.keyboardHook ? 'green' : 'orange'}>
              {t('permissions.keyboardHook')}: {t(status.keyboardHook ? 'permissions.connected' : 'permissions.waiting')}
            </Tag>
          </Space>
          {!status.ready && <>
            <Space wrap>
              {!status.accessibility && <Button loading={pending === 'accessibility'} onClick={() => request('accessibility')}>
                {t('permissions.openAccessibility')}
              </Button>}
              {!status.inputMonitoring && <Button loading={pending === 'inputMonitoring'} onClick={() => request('inputMonitoring')}>
                {t('permissions.openInputMonitoring')}
              </Button>}
              <Button onClick={refresh}>{t('permissions.refresh')}</Button>
              <Button onClick={() => relaunch().catch(err => setError(String(err)))}>{t('permissions.restart')}</Button>
            </Space>
            <Typography.Text type="secondary">{t('permissions.recovery')}</Typography.Text>
            <Typography.Text style={{ overflowWrap: 'anywhere' }}>{status.appPath}</Typography.Text>
          </>}
          {error && <Typography.Text type="danger">{error}</Typography.Text>}
        </Space>
      }
    />
  );
}

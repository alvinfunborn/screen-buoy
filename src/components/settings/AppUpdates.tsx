import { useEffect, useRef, useState } from 'react';
import { Alert, Button, Progress, Space } from 'antd';
import { DownloadOutlined, GithubOutlined, ReloadOutlined } from '@ant-design/icons';
import { getVersion } from '@tauri-apps/api/app';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { useTranslation } from 'react-i18next';
import { SettingSection } from './SettingSection';

interface ReleaseStatus {
  currentVersion: string;
  version: string;
  notes: string;
  newer: boolean;
  canInstall: boolean;
}
type Phase = 'idle' | 'checking' | 'latest' | 'available' | 'downloading' | 'installing' | 'ready' | 'error';

export function AppUpdates() {
  const { t } = useTranslation();
  const [version, setVersion] = useState('…');
  const [release, setRelease] = useState<ReleaseStatus>();
  const [phase, setPhase] = useState<Phase>('idle');
  const [error, setError] = useState('');
  const [progress, setProgress] = useState<number>();
  const update = useRef<Update>();
  const busy = useRef(false);
  const mounted = useRef(false);
  const working = ['checking', 'downloading', 'installing'].includes(phase);

  useEffect(() => {
    mounted.current = true;
    void getVersion().then(value => { if (mounted.current) setVersion(value); })
      .catch(cause => { if (mounted.current) setError(String(cause)); });
    return () => {
      mounted.current = false;
      // Do not close an update resource while its download/install is active.
      if (!busy.current) void update.current?.close().catch(() => {});
    };
  }, []);

  const checkForUpdates = async () => {
    if (busy.current) return;
    busy.current = true;
    setPhase('checking');
    setError('');
    setRelease(undefined);
    try {
      await update.current?.close();
      update.current = undefined;
      const latest = await invoke<ReleaseStatus>('get_release_status');
      setVersion(latest.currentVersion);
      if (latest.newer && latest.canInstall) {
        const candidate = await check({ timeout: 15000 });
        if (!candidate) throw new Error(t('updates.manifestUnavailable'));
        update.current = candidate;
        latest.version = candidate.version;
        latest.notes = latest.notes || candidate.body || '';
      }
      if (mounted.current) {
        setRelease(latest);
        setPhase(latest.newer ? 'available' : 'latest');
      }
    } catch (cause) {
      if (mounted.current) {
        setError(String(cause));
        setPhase('error');
      }
    } finally {
      busy.current = false;
      if (!mounted.current) void update.current?.close().catch(() => {});
    }
  };

  const install = async () => {
    if (busy.current || !update.current) return;
    busy.current = true;
    setPhase('downloading');
    setError('');
    setProgress(undefined);
    let downloaded = 0;
    let total: number | undefined;
    try {
      await update.current.download(event => {
        if (event.event === 'Started') total = event.data.contentLength;
        if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          if (mounted.current && total) setProgress(Math.min(100, Math.floor(downloaded / total * 100)));
        }
      }, { timeout: 120000 });
      if (mounted.current) setPhase('installing');
      await update.current.install();
      // On Windows the installer exits the running app. On macOS, allow the
      // user to restart when ready, including retrying a failed relaunch.
      if (mounted.current) setPhase('ready');
    } catch (cause) {
      if (mounted.current) {
        setError(String(cause));
        setPhase('error');
      }
    } finally {
      await update.current?.close().catch(() => {});
      update.current = undefined;
      busy.current = false;
    }
  };

  const restart = async () => {
    try { await relaunch(); } catch (cause) { setError(String(cause)); }
  };
  const openReleases = async () => {
    try { await invoke('open_releases_page'); } catch (cause) { setError(String(cause)); }
  };

  return (
    <SettingSection title={t('updates.title')} description={t('updates.description')}>
      <div className="app-updates" aria-live="polite">
        <div className="update-actions">
          <span>{t('updates.currentVersion', { version })}</span>
          <Space wrap>
            {phase !== 'ready' && <Button icon={<ReloadOutlined />} loading={phase === 'checking'} disabled={working}
              onClick={() => void checkForUpdates()}>{t('updates.check')}</Button>}
            <Button icon={<GithubOutlined />} disabled={working} onClick={() => void openReleases()}>{t('updates.releases')}</Button>
          </Space>
        </div>
        {phase === 'latest' && <Alert type="success" showIcon message={t('updates.latest')} />}
        {phase === 'available' && release && <>
          <Alert type="info" showIcon message={t('updates.available', { version: release.version })}
            description={t(release.canInstall ? 'updates.installHelp' : 'updates.manualHelp')} />
          {release.canInstall && <Button type="primary" icon={<DownloadOutlined />} onClick={() => void install()}>{t('updates.install')}</Button>}
        </>}
        {(phase === 'downloading' || phase === 'installing') && <div>
          <p>{t(phase === 'downloading' ? 'updates.downloading' : 'updates.installing')}</p>
          <Progress percent={progress} status="active" showInfo={progress !== undefined} />
        </div>}
        {phase === 'ready' && <Alert type="success" showIcon message={t('updates.ready')}
          action={<Button type="primary" onClick={() => void restart()}>{t('updates.restart')}</Button>} />}
        {error && <Alert type="error" showIcon message={t('updates.failed')}
          description={<details><summary>{t('updates.details')}</summary><pre>{error}</pre></details>} />}
        {release?.newer && release.notes && <details className="update-notes">
          <summary>{t('updates.notes')}</summary><pre>{release.notes}</pre>
        </details>}
      </div>
    </SettingSection>
  );
}

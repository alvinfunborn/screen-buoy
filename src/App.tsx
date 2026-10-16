import { useState, useEffect } from 'react'
import Settings from './components/settings/Settings'
import { ConfigProvider, message } from 'antd'
import enUS from 'antd/locale/en_US'
import zhCN from 'antd/locale/zh_CN'
import { useTranslation } from 'react-i18next'
import 'antd/dist/reset.css'
import './styles/App.css'
import { listen } from '@tauri-apps/api/event'


function App() {
  const { i18n } = useTranslation()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const handleError = (error: ErrorEvent) => {
      console.error('React Error:', error);
      setError(error.message);
    };
    window.addEventListener('error', handleError);

    const unlistenPanic = listen('rust-panic', (event) => {
      console.error('Rust panic:', event.payload);
      message.error({
        content: 'program panic, please check the console for details',
        duration: 0,
      });
    });

    return () => {
      window.removeEventListener('error', handleError);
      unlistenPanic.then(unlisten => unlisten());
    };
  }, []);

  return (
    <ConfigProvider locale={i18n.resolvedLanguage === 'zh' ? zhCN : enUS} theme={{
      token: {
        colorPrimary: '#6941c6', colorInfo: '#6941c6', colorSuccess: '#247f64',
        colorText: '#292633', colorTextSecondary: '#706b7c', colorBorder: '#dfdce6',
        colorBgLayout: '#f8f7fa', borderRadius: 8, controlHeight: 36, fontSize: 13,
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Microsoft YaHei", sans-serif',
      },
      components: {
        Tabs: { itemSelectedColor: '#5931b3', itemHoverColor: '#5931b3', titleFontSize: 13 },
        Select: { multipleItemBg: '#f0ecf8' },
        Form: { labelColor: '#3b3648', labelFontSize: 13 },
        Collapse: { headerBg: '#faf9fc', contentBg: '#ffffff' },
      },
    }}>
    <div className="container">
      {error && (
        <div className="error-message">
          Error: {error}
        </div>
      )}
      <Settings />
    </div>
    </ConfigProvider>
  )
}

export default App
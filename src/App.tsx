import { useState, useEffect } from 'react'
import Settings from './components/settings/Settings'
import { ConfigProvider, message } from 'antd'
import en_US from 'antd/locale/en_US'
import zh_CN from 'antd/locale/zh_CN'
import zh_TW from 'antd/locale/zh_TW'
import ja_JP from 'antd/locale/ja_JP'
import ko_KR from 'antd/locale/ko_KR'
import es_ES from 'antd/locale/es_ES'
import fr_FR from 'antd/locale/fr_FR'
import de_DE from 'antd/locale/de_DE'
import pt_BR from 'antd/locale/pt_BR'
import ru_RU from 'antd/locale/ru_RU'
import it_IT from 'antd/locale/it_IT'
import ar_EG from 'antd/locale/ar_EG'
import hi_IN from 'antd/locale/hi_IN'
import id_ID from 'antd/locale/id_ID'
import vi_VN from 'antd/locale/vi_VN'
import th_TH from 'antd/locale/th_TH'
import tr_TR from 'antd/locale/tr_TR'
import { languages } from './language.js'
import { useTranslation } from 'react-i18next'
import 'antd/dist/reset.css'
import './styles/App.css'
import { listen } from '@tauri-apps/api/event'


const componentLocales = {
  'en': en_US,
  'zh': zh_CN,
  'zh-TW': zh_TW,
  'ja': ja_JP,
  'ko': ko_KR,
  'es': es_ES,
  'fr': fr_FR,
  'de': de_DE,
  'pt': pt_BR,
  'ru': ru_RU,
  'it': it_IT,
  'ar': ar_EG,
  'hi': hi_IN,
  'id': id_ID,
  'vi': vi_VN,
  'th': th_TH,
  'tr': tr_TR,
}

function App() {
  const { t, i18n } = useTranslation()
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
        content: i18n.t('settings.panicError'),
        duration: 0,
      });
    });

    return () => {
      window.removeEventListener('error', handleError);
      unlistenPanic.then(unlisten => unlisten());
    };
  }, []);

  return (
    <ConfigProvider locale={componentLocales[i18n.resolvedLanguage as keyof typeof componentLocales] ?? en_US}
      direction={languages.find(language => language.code === i18n.resolvedLanguage)?.direction === 'rtl' ? 'rtl' : 'ltr'} theme={{
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
          {t('settings.runtimeError', { error })}
        </div>
      )}
      <Settings />
    </div>
    </ConfigProvider>
  )
}

export default App
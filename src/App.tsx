import { useState, useEffect } from 'react'
import Settings from './components/settings/Settings'
import { message, Typography } from 'antd'
import 'antd/dist/reset.css'
import './styles/App.css'
import { listen } from '@tauri-apps/api/event'

const { Title, Text } = Typography

function App() {
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
    <div className="container">
      <Title level={3} style={{ marginTop: '24px', marginBottom: '8px', fontWeight: 'bold' }}>
        Settings
      </Title>
      <Text type="secondary" style={{ display: 'block' }}>
        Restart to apply changes
      </Text>
      {error && (
        <div className="error-message">
          错误: {error}
        </div>
      )}
      <Settings />
    </div>
  )
}

export default App
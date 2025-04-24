import { useState, useEffect } from 'react'
import Settings from './components/settings/Settings'
import { Typography } from 'antd'
import 'antd/dist/reset.css'
import './styles/App.css'

const { Title, Text } = Typography

function App() {
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const handleError = (error: ErrorEvent) => {
      console.error('React Error:', error);
      setError(error.message);
    };

    window.addEventListener('error', handleError);
    return () => {
      window.removeEventListener('error', handleError);
    };
  }, []);

  return (
    <div>
      <Title level={3} style={{ marginTop: '24px', marginBottom: '8px', padding: '0 20px' }}>
        Settings
      </Title>
      <Text type="secondary" style={{ display: 'block', padding: '0 20px' }}>
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
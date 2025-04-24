import { useState, useEffect } from 'react'
import Settings from './components/settings/Settings'
import 'antd/dist/reset.css'

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
    <div className="container">
      <h1>Screen Buoy Settings</h1>
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
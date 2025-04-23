import { useState, useEffect } from 'react'
import './styles/App.css'

function App() {
  const [showHints, setShowHints] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isScanning, setIsScanning] = useState(false)
  const [isActive, setIsActive] = useState(false)

  useEffect(() => {
    // 添加错误处理
    const handleError = (error: ErrorEvent) => {
      console.error('React Error:', error);
      setError(error.message);
    };

    window.addEventListener('error', handleError);

    return () => {
      window.removeEventListener('error', handleError);
    };
  }, []);

  // 测试日志输出
  useEffect(() => {
    console.log('App组件已加载');
    console.warn('这是一个警告消息');
    console.error('这是一个错误消息');
  }, []);

  return (
    <div className="container">
      <h1>屏幕元素探测器 🔍</h1>

      <div className="row">
        <button
          disabled={loading}
          className="primary-button"
          onClick={() => {
            console.log('点击了获取元素按钮');
          }}
        >
          {loading ? '加载中...' : '获取所有可见元素'}
        </button>

        <button
          className="primary-button hint-button"
          onClick={() => {
            console.log('点击了提示按钮');
            setShowHints(!showHints);
          }}
        >
          {showHints ? '隐藏提示' : '显示提示'}
        </button>

        <button
          type="button"
          className="primary-button"
          onClick={() => {
            console.log('点击了开发者工具按钮');
          }}
        >
          打开 Overlay 开发者工具
        </button>
      </div>

      {error && (
        <div className="error-message">
          错误: {error}
        </div>
      )}

      <div className="elements-container">
        <div className="elements-section">
          <div className="elements-list">
          </div>
        </div>

        <div className="elements-section">
          <div className="elements-list">
          </div>
        </div>
      </div>

    </div>
  )
}

export default App
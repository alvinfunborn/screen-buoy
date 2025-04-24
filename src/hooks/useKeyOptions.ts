import { Form } from 'antd';
import { useMemo } from 'react';
import type { Config } from '../types/config';

export const useKeyOptions = (form: ReturnType<typeof Form.useForm<Config>>[0]) => {
  // 使用 useWatch 监听 keyboard.available_key 的变化
  const availableKeys = Form.useWatch('keyboard.available_key', form) as Record<string, number> | undefined;
  
  // 动态计算 keyOptions
  return useMemo(() => {
    const options = availableKeys ? 
      Object.entries(availableKeys)
        .sort((a, b) => a[1] - b[1])
        .map(([key, _]) => ({
          label: key,
          value: key
        })) : [];
        
    // 添加固定的选项
    return [
      ...options,
      { label: "HintKey", value: "HintKey" },
      { label: "HintRightKey", value: "HintRightKey" },
      { label: "HintLeftKey", value: "HintLeftKey" }
    ];
  }, [availableKeys]);
}; 
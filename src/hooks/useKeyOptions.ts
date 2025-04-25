import { Form } from 'antd';
import { useMemo } from 'react';
import type { Config } from '../types/config';

// Hook now accepts availableKeys data directly
export const useKeyOptions = (availableKeysData: Record<string, number> | undefined) => {

  // 动态计算 keyOptions, based on the passed prop
  return useMemo(() => {
    console.log('[useKeyOptions] Recalculating. availableKeysData:', availableKeysData);

    const options = availableKeysData ?
      Object.entries(availableKeysData)
        .sort((a, b) => a[1] - b[1]) // 按 virtualKey 排序
        .map(([key, _]) => ({
          label: key,
          value: key
        })) : [];

    return [
      ...options,
      { label: "HintKey", value: "HintKey" },
      { label: "HintRightKey", value: "HintRightKey" },
      { label: "HintLeftKey", value: "HintLeftKey" }
    ];

  }, [availableKeysData]); // Depend on the prop
}; 
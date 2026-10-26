import { useEffect, useState } from 'react';
import { Input } from 'antd';
import type { InputProps } from 'antd';
import { parseCharsetInput } from './hint-validation.js';

type CharsetInputProps = Omit<InputProps, 'value' | 'onChange' | 'defaultValue'> & {
  value?: string[];
  onChange?: (characters: string[]) => void;
};

// Keep the form value an array on every edit, while preserving typed commas
// and spaces so autosave never receives a transient string or moves the caret.
export function CharsetInput({ value, onChange, ...props }: CharsetInputProps) {
  const [raw, setRaw] = useState(() => (value ?? []).join(', '));
  useEffect(() => {
    setRaw(previous => JSON.stringify(parseCharsetInput(previous)) === JSON.stringify(value ?? [])
      ? previous : (value ?? []).join(', '));
  }, [value]);
  return <Input {...props} value={raw} onChange={event => {
    const next = event.target.value;
    setRaw(next);
    onChange?.(parseCharsetInput(next));
  }} />;
}

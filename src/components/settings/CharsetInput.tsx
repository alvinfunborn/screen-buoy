import { Input } from 'antd';
import type { InputProps } from 'antd';
import { parseCharsetInput } from './hint-validation.js';

type CharsetInputProps = Omit<InputProps, 'value' | 'onChange' | 'defaultValue'> & {
  value?: string[];
  onChange?: (characters: string[]) => void;
};

// Display the stored characters directly. Every code point is a member,
// including punctuation and spaces; no character acts as a separator.
export function CharsetInput({ value, onChange, ...props }: CharsetInputProps) {
  return <Input {...props} value={(value ?? []).join('')} onChange={event => {
    onChange?.(parseCharsetInput(event.target.value));
  }} />;
}

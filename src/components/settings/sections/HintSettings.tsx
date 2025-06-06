import React, { useState, useEffect } from 'react';
import { Form, Space, Typography, Spin, Input, Button, InputNumber, Collapse, Select } from 'antd';
import type { NamePath } from 'antd/es/form/interface';
import type { Config, HintType } from '../../../types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import '../../../styles/global.css';
import { useTranslation } from 'react-i18next';

const { Title, Paragraph } = Typography;

interface HintSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

// Helper to generate a unique enough key for new items
const generateTempKey = () => `new_type_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;

export const HintSettings: React.FC<HintSettingsProps> = ({ onValuesChange }) => {
  const { t } = useTranslation();
  const form = Form.useFormInstance<Config>(); // Get form instance from context

  // States for raw input values (comma-separated strings)
  const [rawStyleInputs, setRawStyleInputs] = useState<Record<string, string>>({});
  const [rawCharsetInputs, setRawCharsetInputs] = useState<Record<number, string>>({});
  const [rawExtraCharsetInput, setRawExtraCharsetInput] = useState<string>('');
  const [rawDefaultStyle, setRawDefaultStyle] = useState<string>('');
  // State for element_control_types raw input
  const [rawElementControlTypesInputs, setRawElementControlTypesInputs] = useState<Record<string, string>>({});

  // State to manage the list of hint type names (keys) currently being displayed/edited
  const [hintTypeNames, setHintTypeNames] = useState<string[]>([]);
  // State for the input field when adding a new hint type name
  const [newTypeNameInput, setNewTypeNameInput] = useState<string>('');

  // 新增：Hint Grid 配置相关 state
  const [rawShowAtRowsInput, setRawShowAtRowsInput] = useState<string>('');
  const [rawShowAtColumnsInput, setRawShowAtColumnsInput] = useState<string>('');

  // Initialize states from form values on mount or when form instance changes
  useEffect(() => {
    const initialHintConfig = form.getFieldValue('hint');
    if (initialHintConfig?.types) {
      const typeNames = Object.keys(initialHintConfig.types);
      setHintTypeNames(typeNames);

      const initialRawStyles = typeNames.reduce((acc, key) => {
        const value = initialHintConfig.types[key];
        const styleValue = typeof value.style === 'string' ? value.style : '';
        acc[key] = styleValue;
        return acc;
      }, {} as Record<string, string>);
      setRawStyleInputs(initialRawStyles);

      const initialRawElementTypes = typeNames.reduce((acc, key) => {
        const value = initialHintConfig.types[key];
        acc[key] = Array.isArray(value.element_control_types) ? value.element_control_types.join(', ') : '';
        return acc;
      }, {} as Record<string, string>);
      setRawElementControlTypesInputs(initialRawElementTypes);
    } else {
      setHintTypeNames([]); // Ensure empty if no types initially
      setRawStyleInputs({});
      setRawElementControlTypesInputs({});
    }

    if (initialHintConfig?.charsets) {
      const initialRawCharsets = initialHintConfig.charsets.reduce((acc: Record<number, string>, value: string[], index: number) => {
        acc[index] = Array.isArray(value) ? value.join(', ') : '';
        return acc;
      }, {} as Record<number, string>);
      setRawCharsetInputs(initialRawCharsets);
    } else {
      setRawCharsetInputs({});
    }

    if (initialHintConfig?.charset_extra) {
      setRawExtraCharsetInput(Array.isArray(initialHintConfig.charset_extra) ? initialHintConfig.charset_extra.join(', ') : '');
    } else {
      setRawExtraCharsetInput('');
    }

    if (initialHintConfig?.style) {
      setRawDefaultStyle(initialHintConfig.style);
    } else {
      setRawDefaultStyle('');
    }

    // useEffect 初始化 grid 字段
    const grid = form.getFieldValue(['hint', 'grid']) || {};
    setRawShowAtRowsInput(Array.isArray(grid.show_at_rows) ? grid.show_at_rows.join(', ') : '');
    setRawShowAtColumnsInput(Array.isArray(grid.show_at_columns) ? grid.show_at_columns.join(', ') : '');
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [form]); // Rerun if form instance changes

  const [gridRows, setGridRows] = useState<number>(form.getFieldValue(['hint', 'grid', 'rows']) || 3);
  const [gridColumns, setGridColumns] = useState<number>(form.getFieldValue(['hint', 'grid', 'columns']) || 3);

  useEffect(() => {
    setGridRows(form.getFieldValue(['hint', 'grid', 'rows']) || 3);
    setGridColumns(form.getFieldValue(['hint', 'grid', 'columns']) || 3);
  }, [form.getFieldValue(['hint', 'grid', 'rows']), form.getFieldValue(['hint', 'grid', 'columns'])]);

  // --- Event Handlers ---

  // Style Input Handlers
  const handleRawStyleChange = (typeName: string, value: string) => {
    setRawStyleInputs(prev => ({ ...prev, [typeName]: value }));
  };

  const handleRawStyleBlur = (typeName: string) => {
    const rawValue = rawStyleInputs[typeName] ?? '';
    const fieldPath: NamePath = ['hint', 'types', typeName, 'style'];
    const valueToSet = rawValue.trim() ? rawValue : undefined; // Set undefined if empty/whitespace

    form.setFieldValue(fieldPath, valueToSet);
    form.validateFields([fieldPath]); // Validate after setting

    if (onValuesChange) {
      const currentHintTypes = form.getFieldValue(['hint', 'types']) || {};
      const updatedHintType = { ...(currentHintTypes[typeName] || {}), style: valueToSet };
      const updatedHintTypes = { ...currentHintTypes, [typeName]: updatedHintType };
      const changedValues = { hint: { types: { [typeName]: { style: valueToSet } } } }; // More specific change
      const allValues = form.getFieldsValue(true);
      onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), types: updatedHintTypes } });
    }
  };

  // Charsets List Handlers
  const handleRawCharsetChange = (fieldKey: number, value: string) => {
    setRawCharsetInputs(prev => ({ ...prev, [fieldKey]: value }));
  };

  const handleRawCharsetBlur = (fieldKey: number, fieldName: number) => {
    const rawValue = rawCharsetInputs[fieldKey] ?? '';
    const newArrayValue = rawValue ? rawValue.split(',').map(s => s.trim()).filter(s => s) : [];
    const fieldPath: NamePath = ['hint', 'charsets', fieldName];
    form.setFieldValue(fieldPath, newArrayValue);

    if (onValuesChange) {
      const currentCharsets = form.getFieldValue(['hint', 'charsets']) || [];
      // Create a new array representing the state after the update
      const updatedCharsets = [...currentCharsets];
      if (fieldName >= updatedCharsets.length) {
        updatedCharsets.length = fieldName + 1; // Ensure array is long enough
      }
      updatedCharsets[fieldName] = newArrayValue; // Place the new value

      const changedValues = { hint: { charsets: updatedCharsets } }; // Reflect the whole array change potentially
      const allValues = form.getFieldsValue(true);
      // Ensure allValues reflects the change we just made
      const updatedAllValues = { ...allValues, hint: { ...(allValues.hint || {}), charsets: updatedCharsets } };
      onValuesChange(changedValues, updatedAllValues);
      form.validateFields([fieldPath]); // Use the path directly
    } else {
      form.validateFields([fieldPath]);
    }
  };

  // Extra Charset Handlers
  const handleRawExtraCharsetChange = (value: string) => {
    setRawExtraCharsetInput(value);
  };

  const handleRawExtraCharsetBlur = () => {
    const rawValue = rawExtraCharsetInput ?? '';
    const newArrayValue = rawValue ? rawValue.split(',').map(s => s.trim()).filter(s => s) : [];
    const fieldPath: NamePath = ['hint', 'charset_extra'];
    form.setFieldValue(fieldPath, newArrayValue);

    if (onValuesChange) {
      const changedValues = { hint: { charset_extra: newArrayValue } };
      const allValues = form.getFieldsValue(true);
      // Ensure allValues reflects the change
      const updatedAllValues = { ...allValues, hint: { ...(allValues.hint || {}), charset_extra: newArrayValue } };
      onValuesChange(changedValues, updatedAllValues);
      form.validateFields([fieldPath]);
    } else {
      form.validateFields([fieldPath]);
    }
  };

  const handleRawDefaultStyleChange = (value: string) => {
    setRawDefaultStyle(value);
  };

  const handleRawDefaultStyleBlur = () => {
    const rawValue = rawDefaultStyle ?? '';
    const fieldPath: NamePath = ['hint', 'style'];
    form.setFieldValue(fieldPath, rawValue);

    if (onValuesChange) {
      const changedValues = { hint: { style: rawValue } };
      const allValues = form.getFieldsValue(true);
      const updatedAllValues = { ...allValues, hint: { ...(allValues.hint || {}), style: rawValue } };
      onValuesChange(changedValues, updatedAllValues);
      form.validateFields([fieldPath]);
    } else {
      form.validateFields([fieldPath]);
    }
  };

  // Element Control Types Handlers (New)
  const handleRawElementControlTypeChange = (typeName: string, value: string) => {
    setRawElementControlTypesInputs(prev => ({ ...prev, [typeName]: value }));
  };

  const handleRawElementControlTypeBlur = (typeName: string) => {
    const rawValue = rawElementControlTypesInputs[typeName] ?? '';
    // Convert comma-separated string to array of numbers, filtering out non-integers
    const newArrayValue = rawValue
      .split(',')                   // Split by comma
      .map(s => s.trim())           // Trim whitespace
      .filter(s => s)               // Filter out empty strings
      .map(s => parseInt(s, 10))   // Parse to integer (base 10)
      .filter(num => !isNaN(num)); // Filter out NaN values (parsing failures)

    const fieldPath: NamePath = ['hint', 'types', typeName, 'element_control_types'];
    form.setFieldValue(fieldPath, newArrayValue); // Set the array of numbers
    form.validateFields([fieldPath]); // Validate after setting

    if (onValuesChange) {
      const currentHintTypes = form.getFieldValue(['hint', 'types']) || {};
      // Ensure the updatedHintType reflects the numeric array
      const updatedHintType = { ...(currentHintTypes[typeName] || {}), element_control_types: newArrayValue };
      const updatedHintTypes = { ...currentHintTypes, [typeName]: updatedHintType };
      // Changed values might be more specific if needed, but sending the updated object is often fine
      const changedValues = { hint: { types: { [typeName]: { element_control_types: newArrayValue } } } };
      const allValues = form.getFieldsValue(true);
      // Ensure allValues reflects the numeric array change
      onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), types: updatedHintTypes } });
    }
  };

  // Hint Type Add/Remove Handlers (New)
  const handleAddType = () => {
    const newTypeName = newTypeNameInput.trim();
    if (!newTypeName) {
      // Optionally show an error message: "Type name cannot be empty."
      console.warn("Hint type name cannot be empty.");
      return;
    }
    if (hintTypeNames.includes(newTypeName) || (form.getFieldValue(['hint', 'types']) || {})[newTypeName]) {
      // Optionally show an error message: "Type name already exists."
      console.warn(`Hint type name "${newTypeName}" already exists.`);
      return;
    }

    const newHintType: HintType = { style: '', z_index: 0, element_control_types: [] }; // Default values

    // 1. Update Form State
    const currentHintTypes = form.getFieldValue(['hint', 'types']) || {};
    const updatedHintTypes = { ...currentHintTypes, [newTypeName]: newHintType };
    form.setFieldValue(['hint', 'types'], updatedHintTypes);

    // 2. Update Local State Management
    setHintTypeNames(prev => [...prev, newTypeName]);
    setRawStyleInputs(prev => ({ ...prev, [newTypeName]: '' }));
    setRawElementControlTypesInputs(prev => ({ ...prev, [newTypeName]: '' }));

    // 3. Clear Input
    setNewTypeNameInput('');

    // 4. Trigger Change Callback
    if (onValuesChange) {
      const changedValues = { hint: { types: { [newTypeName]: newHintType } } }; // Changed value is the new type added
      const allValues = form.getFieldsValue(true); // Get potentially updated values
      // Ensure allValues reflects the addition we just made
      const updatedAllValues = { ...allValues, hint: { ...(allValues.hint || {}), types: updatedHintTypes } };
      onValuesChange(changedValues, updatedAllValues);
    }

    // Optionally, scroll to the new element or focus its first field
  };

  const handleRemoveType = (typeNameToRemove: string) => {
    // 1. Update Form State
    const currentHintTypes = form.getFieldValue(['hint', 'types']) || {};
    const updatedHintTypes = { ...currentHintTypes };
    delete updatedHintTypes[typeNameToRemove];
    form.setFieldValue(['hint', 'types'], updatedHintTypes);

    // 2. Update Local State Management
    setHintTypeNames(prev => prev.filter(name => name !== typeNameToRemove));
    setRawStyleInputs(prev => {
      const newState = { ...prev };
      delete newState[typeNameToRemove];
      return newState;
    });
    setRawElementControlTypesInputs(prev => {
      const newState = { ...prev };
      delete newState[typeNameToRemove];
      return newState;
    });

    // 3. Trigger Change Callback
    if (onValuesChange) {
      // Indicate that the type was removed. Sending the whole updated object might be simplest.
      const changedValues = { hint: { types: updatedHintTypes } }; // Reflect the state after removal
      const allValues = form.getFieldsValue(true);
      // Ensure allValues reflects the removal
      const updatedAllValues = { ...allValues, hint: { ...(allValues.hint || {}), types: updatedHintTypes } };
      onValuesChange(changedValues, updatedAllValues);
    }
    // Note: We don't need to validate fields of removed items.
  };

  // 失焦时 parse 并 setFieldValue
  const handleShowAtRowsBlur = () => {
    const arr = rawShowAtRowsInput
      .split(',')
      .map(s => parseInt(s.trim(), 10))
      .filter(n => !isNaN(n));
    form.setFieldValue(['hint', 'grid', 'show_at_rows'], arr);
    if (onValuesChange) {
      const changedValues = { hint: { grid: { show_at_rows: arr } } };
      const allValues = form.getFieldsValue(true);
      onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), grid: { ...(allValues.hint?.grid || {}), show_at_rows: arr } } });
    }
  };
  const handleShowAtColumnsBlur = () => {
    const arr = rawShowAtColumnsInput
      .split(',')
      .map(s => parseInt(s.trim(), 10))
      .filter(n => !isNaN(n));
    form.setFieldValue(['hint', 'grid', 'show_at_columns'], arr);
    if (onValuesChange) {
      const changedValues = { hint: { grid: { show_at_columns: arr } } };
      const allValues = form.getFieldsValue(true);
      onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), grid: { ...(allValues.hint?.grid || {}), show_at_columns: arr } } });
    }
  };

  const handleGridRowsChange = (value: number | null) => {
    if (typeof value !== 'number' || isNaN(value)) return;
    setGridRows(value);
    form.setFieldValue(['hint', 'grid', 'rows'], value);
    // 修正 show_at_rows
    const current = form.getFieldValue(['hint', 'grid', 'show_at_rows']) || [];
    const filtered = current.filter((n: number) => n >= 1 && n <= value);
    form.setFieldValue(['hint', 'grid', 'show_at_rows'], filtered);
    if (onValuesChange) {
      const changedValues = { hint: { grid: { rows: value, show_at_rows: filtered } } };
      const allValues = form.getFieldsValue(true);
      onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), grid: { ...(allValues.hint?.grid || {}), rows: value, show_at_rows: filtered } } });
    }
  };
  const handleGridColumnsChange = (value: number | null) => {
    if (typeof value !== 'number' || isNaN(value)) return;
    setGridColumns(value);
    form.setFieldValue(['hint', 'grid', 'columns'], value);
    // 修正 show_at_columns
    const current = form.getFieldValue(['hint', 'grid', 'show_at_columns']) || [];
    const filtered = current.filter((n: number) => n >= 1 && n <= value);
    form.setFieldValue(['hint', 'grid', 'show_at_columns'], filtered);
    if (onValuesChange) {
      const changedValues = { hint: { grid: { columns: value, show_at_columns: filtered } } };
      const allValues = form.getFieldsValue(true);
      onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), grid: { ...(allValues.hint?.grid || {}), columns: value, show_at_columns: filtered } } });
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      {/* Hint Charsets Section (Existing) */}
      <Form.Item label={t('hint.charsets')} className="config-section-title" tooltip={t('hint.charsetsTooltip')}>
        <Form.List name={['hint', 'charsets']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              {fields.map((field, index) => {
                const { key, name, ...restField } = field;
                // Use field.key for managing raw input state map
                const fieldKey = field.key;
                return (
                  <Space key={fieldKey} align="baseline" style={{ width: '100%' }}>
                    <Form.Item
                      {...restField}
                      name={name} // Use field.name for AntD Form binding
                      validateTrigger={['onChange', 'onBlur']}
                      rules={[{ required: true, message: 'Please enter the charset' }]}
                      style={{ flex: 1 }}
                    >
                      <Input
                        placeholder="Enter the charset, separated by commas"
                        // Ensure value defaults to empty string if state entry is missing
                        value={rawCharsetInputs[fieldKey] ?? ''}
                        style={{ width: '400px' }}
                        onChange={(e) => handleRawCharsetChange(fieldKey, e.target.value)}
                        onBlur={() => handleRawCharsetBlur(fieldKey, name)}
                      />
                    </Form.Item>
                    <MinusCircleOutlined style={{ marginLeft: 8 }} onClick={() => remove(field.name)} />
                  </Space>
                );
              })}
              <Button type="dashed" onClick={() => add([])} block icon={<PlusOutlined />}>
                {t('hint.addCharset')}
              </Button>
            </Space>
          )}
        </Form.List>
      </Form.Item>

      {/* Hint Charset Extra Section */}
      <Form.Item
        label={t('hint.charsetExtra')}
        className="config-section-title"
        tooltip={t('hint.charsetExtraTooltip')}
        name={['hint', 'charset_extra']}
      >
        <Input
          placeholder={t('hint.placeholderCharsetExtra')}
          // Value comes from local raw state
          value={rawExtraCharsetInput}
          onChange={(e) => handleRawExtraCharsetChange(e.target.value)}
          // Commit to form state on blur
          onBlur={handleRawExtraCharsetBlur}
        />
      </Form.Item>

      {/* Hint Grid Section */}
      <Paragraph className="config-section-title">{t('hint.grid')}</Paragraph>
      <Space direction="vertical" style={{ width: '100%' }}>
        <Form.Item
          label={t('hint.rows')}
          tooltip={t('hint.rowsTooltip')}
          name={['hint', 'grid', 'rows']}
          style={{ marginBottom: 8 }}
        >
          <InputNumber min={0} style={{ width: 100 }} />
        </Form.Item>
        <Form.Item
          label={t('hint.columns')}
          tooltip={t('hint.columnsTooltip')}
          name={['hint', 'grid', 'columns']}
          style={{ marginBottom: 8 }}
        >
          <InputNumber min={0} style={{ width: 100 }} />
        </Form.Item>
        <Form.Item label={t('hint.showAtRows')} tooltip={t('hint.showAtRowsTooltip')} style={{ marginBottom: 8 }}>
          <Select
            mode="multiple"
            value={form.getFieldValue(['hint', 'grid', 'show_at_rows']) || []}
            options={Array.from({ length: gridRows }, (_, i) => ({ value: i + 1, label: String(i + 1) }))}
            onChange={vals => {
              form.setFieldValue(['hint', 'grid', 'show_at_rows'], vals);
              if (onValuesChange) {
                const changedValues = { hint: { grid: { show_at_rows: vals } } };
                const allValues = form.getFieldsValue(true);
                onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), grid: { ...(allValues.hint?.grid || {}), show_at_rows: vals } } });
              }
            }}
            style={{ width: 400 }}
            placeholder={t('hint.placeholderRows')}
          />
        </Form.Item>
        <Form.Item label={t('hint.showAtColumns')} tooltip={t('hint.showAtColumnsTooltip')} style={{ marginBottom: 8 }}>
          <Select
            mode="multiple"
            value={form.getFieldValue(['hint', 'grid', 'show_at_columns']) || []}
            options={Array.from({ length: gridColumns }, (_, i) => ({ value: i + 1, label: String(i + 1) }))}
            onChange={vals => {
              form.setFieldValue(['hint', 'grid', 'show_at_columns'], vals);
              if (onValuesChange) {
                const changedValues = { hint: { grid: { show_at_columns: vals } } };
                const allValues = form.getFieldsValue(true);
                onValuesChange(changedValues, { ...allValues, hint: { ...(allValues.hint || {}), grid: { ...(allValues.hint?.grid || {}), show_at_columns: vals } } });
              }
            }}
            style={{ width: 400 }}
            placeholder={t('hint.placeholderColumns')}
          />
        </Form.Item>
        <Form.Item
          label={t('hint.hintType')}
          name={['hint', 'grid', 'hint_type']}
          tooltip={t('hint.hintTypeTooltip')}
          style={{ marginBottom: 0 }}
        >
          <Select
            options={hintTypeNames.map(name => ({ value: name, label: name }))}
            style={{ width: 200 }}
            placeholder={t('hint.hintType')}
          />
        </Form.Item>
      </Space>

      {/* Default Style Section */}
      <Form.Item
        label={t('hint.defaultStyle')}
        className="config-section-title"
        tooltip={t('hint.defaultStyleTooltip')}
        name={['hint', 'style']}
      >
        <Input.TextArea
          rows={10}
          placeholder={t('hint.placeholderStyle')}
          value={rawDefaultStyle}
          onChange={(e) => handleRawDefaultStyleChange(e.target.value)}
          onBlur={handleRawDefaultStyleBlur}
        />
      </Form.Item>

      <Paragraph className="config-section-title">{t('hint.types')}</Paragraph>
      {/* Dynamic Hint Types Section - Render based on hintTypeNames state using Collapse */}
      <Collapse accordion style={{ width: '100%', marginBottom: '16px' }}>
        {hintTypeNames.map((typeName) => (
          <Collapse.Panel
            key={typeName}
            header={`Type: ${typeName}`}
            extra={ // Move remove button to panel extra
              <Button
                danger
                size="small"
                onClick={(e) => { e.stopPropagation(); handleRemoveType(typeName); }} // Prevent collapse toggle on button click
                icon={<MinusCircleOutlined />}
                aria-label={`Remove ${typeName}`}
              >
                Remove
              </Button>}
          >
            {/* The existing Space containing Form Items */}
            <Space key={typeName} direction="vertical" style={{ width: '100%' }}>
              {/* Removed the div wrapper for header/button */}
              <Form.Item
                label={t('hint.styleCss')}
                name={['hint', 'types', typeName, 'style']}
                tooltip={t('hint.styleCssTooltip')}
                rules={[{ required: false }]}
                style={{ width: '100%', marginBottom: '10px' }}
              >
                <Input.TextArea
                  rows={6}
                  placeholder={t('hint.placeholderTypeStyle', { typeName })}
                  value={rawStyleInputs[typeName] ?? ''}
                  onChange={(e) => handleRawStyleChange(typeName, e.target.value)}
                  onBlur={() => handleRawStyleBlur(typeName)}
                  style={{ width: '100%', maxWidth: '400px' }}
                />
              </Form.Item>
              <Form.Item
                label={t('hint.zIndex')}
                name={['hint', 'types', typeName, 'z_index']}
                tooltip={t('hint.zIndexTooltip')}
                rules={[{ required: true, type: 'number', message: 'Please enter a z-index' }]}
                style={{ width: '100%', marginBottom: '10px' }}
              >
                <InputNumber
                  placeholder={t('hint.zIndex')}
                  style={{ width: '100px' }}
                />
              </Form.Item>
              <Form.Item
                label={t('hint.elementControlTypes')}
                tooltip={t('hint.elementControlTypesTooltip')}
                style={{ width: '100%', marginBottom: '0px' }}
              >
                <Input
                  placeholder={t('hint.placeholderElementTypes')}
                  value={rawElementControlTypesInputs[typeName] ?? ''}
                  onChange={(e) => handleRawElementControlTypeChange(typeName, e.target.value)}
                  onBlur={() => handleRawElementControlTypeBlur(typeName)}
                  style={{ width: '100%', maxWidth: '300px' }}
                />
              </Form.Item>
            </Space>
          </Collapse.Panel>
        ))}
      </Collapse>

      {/* Add New Hint Type Section */}
      <Space style={{ marginBottom: '32px' }}>
        <Input
          placeholder={t('hint.placeholderNewType')}
          value={newTypeNameInput}
          onChange={(e) => setNewTypeNameInput(e.target.value)}
          style={{ width: '200px' }}
        />
        <Button type="dashed" onClick={handleAddType} icon={<PlusOutlined />}>
          {t('hint.addHintType')}
        </Button>
      </Space>

    </Space>
  );
}; 
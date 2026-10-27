import React, { useState, useEffect } from 'react';
import { Alert, Form, Space, Input, Button, InputNumber, Collapse, Select } from 'antd';
import type { NamePath } from 'antd/es/form/interface';
import type { Config, HintType } from '../../../types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import { SettingSection } from '../SettingSection';
import { useTranslation } from 'react-i18next';
import { CharsetInput } from '../CharsetInput';
import { getHintPrefixConflicts, getHintSideKeyWarnings } from '../hint-validation.js';


interface HintSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
  isMac: boolean;
  axRoles: string[];
}

// Helper to generate a unique enough key for new items
const generateTempKey = () => `new_type_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;

export const HintSettings: React.FC<HintSettingsProps> = ({ onValuesChange, isMac, axRoles }) => {
  const { t } = useTranslation();
  const form = Form.useFormInstance<Config>(); // Get form instance from context

  const charsets = Form.useWatch(['hint', 'charsets'], form) ?? form.getFieldValue(['hint', 'charsets']);
  const charsetExtra = Form.useWatch(['hint', 'charset_extra'], form) ?? form.getFieldValue(['hint', 'charset_extra']);
  const sideKeyMapping = Form.useWatch(['keyboard', 'map_left_right'], { form, preserve: true })
    ?? form.getFieldValue(['keyboard', 'map_left_right']);
  const conflicts = getHintPrefixConflicts({ charsets, charset_extra: charsetExtra });
  const conflictMessage = conflicts.length
    ? t('hint.charsetPrefixConflict', { characters: conflicts.join(', ') }) : undefined;
  const sideKeyWarnings = getHintSideKeyWarnings({ charsets }, sideKeyMapping);

  // States for raw input values (comma-separated strings)
  const [rawStyleInputs, setRawStyleInputs] = useState<Record<string, string>>({});
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
    if (isMac) newHintType.element_ax_roles = [];

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
    <div className="settings-stack">
      <SettingSection title={t('hint.charsets')} description={t('hint.charsetsTooltip')}>
      {/* Hint Charsets Section (Existing) */}
      <Form.Item label={t('settings.groups.characterRows')}>
        <Form.List name={['hint', 'charsets']}>
          {(fields, { add, remove }) => (
            <Space direction="vertical" style={{ width: '100%' }}>
              {fields.map((field, index) => {
                const { key, name, ...restField } = field;
                const fieldKey = field.key;
                return (
                  <div key={fieldKey} className="charset-row">
                    <Form.Item
                      {...restField}
                      name={name} // Use field.name for AntD Form binding
                      validateTrigger={['onChange', 'onBlur']}
                      rules={[{ required: true, message: t('hint.placeholderCharset') }]}
                      validateStatus={index === 0 && conflictMessage ? 'error' : undefined}
                      help={index === 0 ? conflictMessage : undefined}
                      style={{ flex: 1 }}
                    >
                      <CharsetInput placeholder={t('hint.placeholderCharset')} style={{ width: '100%' }} />
                    </Form.Item>
                    <Button type="text" danger icon={<MinusCircleOutlined />}
                      aria-label={t('settings.removeItem', { name: `${t('hint.charsets')} ${index + 1}` })}
                      onClick={() => remove(field.name)} />
                  </div>
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
        validateStatus={conflictMessage ? 'error' : undefined}
        help={conflictMessage}
      >
        <CharsetInput placeholder={t('hint.placeholderCharsetExtra')} />
      </Form.Item>

      {sideKeyWarnings.length > 0 && <Alert type="warning" showIcon style={{ marginBottom: 16 }}
        message={t('hint.charsetSideKeyWarning', { characters: sideKeyWarnings.join(', ') })} />}
      </SettingSection>
      <SettingSection title={t('hint.grid')} description={t('settings.groups.gridDescription')}>
      <div className="hint-grid-fields">
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
            style={{ width: '100%' }}
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
            style={{ width: '100%' }}
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
            style={{ width: '100%' }}
            placeholder={t('hint.hintType')}
          />
        </Form.Item>
      </div>
      </SettingSection>

      <SettingSection title={t('hint.defaultStyle')} description={t('hint.defaultStyleTooltip')}>
      {/* Default Style Section */}
      <Form.Item
        className="code-field"
        name={['hint', 'style']}
      >
        <Input.TextArea
          rows={7}
          aria-label={t('hint.defaultStyle')}
          spellCheck={false}
          placeholder={t('hint.placeholderStyle')}
          value={rawDefaultStyle}
          onChange={(e) => handleRawDefaultStyleChange(e.target.value)}
          onBlur={handleRawDefaultStyleBlur}
        />
      </Form.Item>

      </SettingSection>
      <SettingSection title={t('hint.types')} description={t(isMac ? 'hint.macTypesDescription' : 'settings.groups.typesDescription')}>
      {/* Dynamic Hint Types Section - Render based on hintTypeNames state using Collapse */}
      <Collapse accordion style={{ width: '100%', marginBottom: '16px' }}>
        {hintTypeNames.map((typeName) => (
          <Collapse.Panel
            key={typeName}
            header={<span className="hint-type-name">{typeName}</span>}
            extra={ // Move remove button to panel extra
              <Button
                danger
                size="small"
                onClick={(e) => { e.stopPropagation(); handleRemoveType(typeName); }} // Prevent collapse toggle on button click
                icon={<MinusCircleOutlined />}
                aria-label={t('settings.removeItem', { name: typeName })}
              >
                {t('hint.remove')}
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
                  spellCheck={false}
                  className="code-input"
                  placeholder={t('hint.placeholderTypeStyle', { typeName })}
                  value={rawStyleInputs[typeName] ?? ''}
                  onChange={(e) => handleRawStyleChange(typeName, e.target.value)}
                  onBlur={() => handleRawStyleBlur(typeName)}
                  style={{ width: '100%' }}
                />
              </Form.Item>
              <Form.Item
                label={t('hint.zIndex')}
                name={['hint', 'types', typeName, 'z_index']}
                tooltip={t('hint.zIndexTooltip')}
                rules={[{ required: true, type: 'number', message: t('hint.zIndex') }]}
                style={{ width: '100%', marginBottom: '10px' }}
              >
                <InputNumber
                  placeholder={t('hint.zIndex')}
                  style={{ width: '100px' }}
                />
              </Form.Item>
              {isMac ? <Form.Item
                label={t('hint.axRoles')}
                tooltip={t('hint.axRolesTooltip')}
                name={['hint', 'types', typeName, 'element_ax_roles']}
                style={{ width: '100%', marginBottom: 0 }}
              >
                <Select mode="tags" options={axRoles.map(role => ({ value: role, label: role }))}
                  tokenSeparators={[',']} placeholder={t('hint.placeholderAxRoles')} />
              </Form.Item> : <Form.Item
                label={t('hint.elementControlTypes')}
                tooltip={t('hint.elementControlTypesTooltip')}
                style={{ width: '100%', marginBottom: '0px' }}
              >
                <Input
                  placeholder={t('hint.placeholderElementTypes')}
                  value={rawElementControlTypesInputs[typeName] ?? ''}
                  onChange={(e) => handleRawElementControlTypeChange(typeName, e.target.value)}
                  onBlur={() => handleRawElementControlTypeBlur(typeName)}
                  style={{ width: '100%' }}
                />
              </Form.Item>}
            </Space>
          </Collapse.Panel>
        ))}
      </Collapse>

      {/* Add New Hint Type Section */}
      <div className="add-type-row">
        <Input
          placeholder={t('hint.placeholderNewType')}
          value={newTypeNameInput}
          onChange={(e) => setNewTypeNameInput(e.target.value)}
          style={{ width: '100%' }}
        />
        <Button type="dashed" onClick={handleAddType} icon={<PlusOutlined />}>
          {t('hint.addHintType')}
        </Button>
      </div>
      </SettingSection>
    </div>
  );
};

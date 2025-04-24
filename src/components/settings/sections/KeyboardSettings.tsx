import React, { useEffect, useState } from 'react';
import { Form, Space, Typography, Button, Input, List, Spin, Collapse, Select, InputNumber } from 'antd';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import type { Config, KeyboardConfig, LeftRightConfig } from '../../../types/config';

const { Title } = Typography;
const { Panel } = Collapse;

interface KeyboardSettingsProps {
  loading?: boolean;
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

// Define types for the array format used for local state management
interface AvailableKeyItem {
  id: number; // Add an id for stable key in React list
  keyName: string;
  virtualKey: number;
}
interface LeftRightMapItem {
  id: number; // Add an id for stable key
  keyName: string;
  left: string | null;
  right: string | null;
}

// Helper function to convert Record to Array
const recordToAvailableKeyList = (record: Record<string, number>): AvailableKeyItem[] => {
  return Object.entries(record)
    .map(([keyName, virtualKey], index) => ({ id: index, keyName, virtualKey }))
    .sort((a, b) => a.virtualKey - b.virtualKey);
};

const recordToLeftRightMapList = (record: Record<string, LeftRightConfig>): LeftRightMapItem[] => {
  return Object.entries(record).map(([keyName, value], index) => ({
    id: index,
    keyName,
    left: value.left ?? null,
    right: value.right ?? null,
  }));
};

// Helper function to convert Array back to Record
const availableKeyListToRecord = (list: AvailableKeyItem[]): Record<string, number> => {
  return list.reduce((acc, { keyName, virtualKey }) => {
    if (keyName) { // Ensure keyName is not empty
        acc[keyName] = virtualKey;
    }
    return acc;
  }, {} as Record<string, number>);
};

const leftRightMapListToRecord = (list: LeftRightMapItem[]): Record<string, LeftRightConfig> => {
  return list.reduce((acc, { keyName, left, right }) => {
     if (keyName) { // Ensure keyName is not empty
        acc[keyName] = { left: left || null, right: right || null };
     }
    return acc;
  }, {} as Record<string, LeftRightConfig>);
};

export const KeyboardSettings: React.FC<KeyboardSettingsProps> = ({ loading, onValuesChange }) => {
  const form = Form.useFormInstance<Config>();
  const [availableKeyList, setAvailableKeyList] = useState<AvailableKeyItem[]>([]);
  const [leftRightMapList, setLeftRightMapList] = useState<LeftRightMapItem[]>([]);

  // Special keys that shouldn't be removed or have their key name changed
  const specialKeys = ["HintKey", "HintRightKey", "HintLeftKey"];

  // Load initial data from form into local state
  useEffect(() => {
    const initialKeyboardConfig = form.getFieldValue('keyboard') as KeyboardConfig;
    if (initialKeyboardConfig && initialKeyboardConfig.available_key && initialKeyboardConfig.map_left_right) {
        setAvailableKeyList(recordToAvailableKeyList(initialKeyboardConfig.available_key));
        setLeftRightMapList(recordToLeftRightMapList(initialKeyboardConfig.map_left_right));
    }
    // Intentionally not depending on the lists themselves to avoid loops on initial set
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [form]); // Rerun if form instance changes

  // Function to update form state whenever local list state changes
  const updateFormState = (newAvailableKeys?: AvailableKeyItem[], newLeftRightMap?: LeftRightMapItem[]) => {
    const currentKeyboardConfig = form.getFieldValue('keyboard') as KeyboardConfig;
    const updatedKeyboardConfig: Partial<KeyboardConfig> = {};
    let changedFieldName: 'available_key' | 'map_left_right' | null = null;

    if (newAvailableKeys) {
        updatedKeyboardConfig.available_key = availableKeyListToRecord(newAvailableKeys);
        changedFieldName = 'available_key';
    }
    if (newLeftRightMap) {
        updatedKeyboardConfig.map_left_right = leftRightMapListToRecord(newLeftRightMap);
         changedFieldName = 'map_left_right';
    }

    const newKeyboardValue = {
        ...currentKeyboardConfig,
        ...updatedKeyboardConfig
    };

    form.setFieldsValue({
        keyboard: newKeyboardValue
    });

    // Manually trigger the parent's onValuesChange after setting the value
    if (onValuesChange && changedFieldName) {
        // Construct the arguments for onValuesChange
        // changedValues should reflect what specifically changed
        const changedValues = { keyboard: { [changedFieldName]: updatedKeyboardConfig[changedFieldName] } };
        // allValues should be the complete, updated form state
        const allValues = form.getFieldsValue(true);
        onValuesChange(changedValues, allValues);
    }
  };

  // --- Handlers for Available Keys ---
  const handleAvailableKeyNameChange = (id: number, newKeyName: string) => {
    const newList = availableKeyList.map(item =>
      item.id === id ? { ...item, keyName: newKeyName.trim() } : item
    );
    setAvailableKeyList(newList);
    updateFormState(newList, undefined);
  };

  const handleAvailableKeyValueChange = (id: number, newVirtualKey: number | null) => {
    const newList = availableKeyList.map(item =>
      item.id === id ? { ...item, virtualKey: newVirtualKey ?? 0 } : item
    );
    setAvailableKeyList(newList);
    updateFormState(newList, undefined);
  };

  const handleAddAvailableKey = () => {
    const newId = availableKeyList.length ? Math.max(...availableKeyList.map(i => i.id)) + 1 : 0;
    const newList = [...availableKeyList, { id: newId, keyName: `NewKey${newId}`, virtualKey: 0 }];
    setAvailableKeyList(newList);
    updateFormState(newList, undefined);
  };

  const handleRemoveAvailableKey = (id: number) => {
    const newList = availableKeyList.filter(item => item.id !== id);
    setAvailableKeyList(newList);
    updateFormState(newList, undefined);
  };

  // --- Handlers for Left Right Map ---
   const handleLeftRightKeyNameChange = (id: number, newKeyName: string) => {
    const newList = leftRightMapList.map(item =>
      item.id === id ? { ...item, keyName: newKeyName.trim() } : item
    );
    setLeftRightMapList(newList);
    updateFormState(undefined, newList);
  };

  const handleLeftRightLeftChange = (id: number, newLeftValue: string) => {
    const newList = leftRightMapList.map(item =>
      item.id === id ? { ...item, left: newLeftValue || null } : item
    );
    setLeftRightMapList(newList);
    updateFormState(undefined, newList);
  };

  const handleLeftRightRightChange = (id: number, newRightValue: string) => {
    const newList = leftRightMapList.map(item =>
      item.id === id ? { ...item, right: newRightValue || null } : item
    );
    setLeftRightMapList(newList);
    updateFormState(undefined, newList);
  };

   const handleAddLeftRightMap = () => {
    const newId = leftRightMapList.length ? Math.max(...leftRightMapList.map(i => i.id)) + 1 : 0;
    const newList = [...leftRightMapList, { id: newId, keyName: `NewMap${newId}`, left: null, right: null }];
    setLeftRightMapList(newList);
    updateFormState(undefined, newList);
  };

  const handleRemoveLeftRightMap = (id: number) => {
    const newList = leftRightMapList.filter(item => item.id !== id);
    setLeftRightMapList(newList);
    updateFormState(undefined, newList);
  };

  if (loading) {
    return <Spin />;
  }

  // Options for Propagation Modifier Select (needs available keys from local state now)
  const keyOptionsForSelect = availableKeyList.map(item => ({
      label: item.keyName,
      value: item.keyName
  }));

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Keyboard Settings</Title>

      {/* Propagation Modifier */}
      <Form.Item
        label="Propagation Modifier"
        name={['keyboard', 'propagation_modifier']}
      >
        <Select
          mode="multiple"
          placeholder="Select modifiers"
          style={{ width: '100%' }}
          options={keyOptionsForSelect} // Use state-derived options
        />
      </Form.Item>

      <Collapse>
        <Panel header="Available Keys" key="available_keys">
          {/* Render list manually based on local state */}
          <List
            bordered
            dataSource={availableKeyList}
            rowKey="id"
            renderItem={(item) => {
              const isSpecial = specialKeys.includes(item.keyName);
              return (
                <List.Item key={item.id}>
                  <Space align="center">
                    <Input
                      addonBefore="Key"
                      value={item.keyName}
                      onChange={(e) => handleAvailableKeyNameChange(item.id, e.target.value)}
                      disabled={isSpecial}
                      style={{ width: '250px' }}
                      placeholder="Key Name"
                    />
                    <InputNumber
                      addonBefore="VK"
                      value={item.virtualKey}
                      onChange={(value) => handleAvailableKeyValueChange(item.id, value)}
                      style={{ width: '150px' }}
                      placeholder="Virtual Key"
                    />
                    {!isSpecial && (
                      <Button
                        danger
                        icon={<MinusCircleOutlined />}
                        onClick={() => handleRemoveAvailableKey(item.id)}
                      />
                    )}
                  </Space>
                </List.Item>
              );
            }}
          />
          <Button
            type="dashed"
            onClick={handleAddAvailableKey}
            block
            icon={<PlusOutlined />}
            style={{ marginTop: '10px' }}
          >
            Add Available Key
          </Button>
        </Panel>
      </Collapse>

      <Collapse>
        <Panel header="Left Right Mapping" key="left_right_mapping">
           {/* Render list manually based on local state */}
          <List
            bordered
            dataSource={leftRightMapList}
             rowKey="id"
            renderItem={(item) => (
              <List.Item key={item.id}>
                <Space align="center">
                   <Input
                      addonBefore="Key"
                      value={item.keyName}
                      onChange={(e) => handleLeftRightKeyNameChange(item.id, e.target.value)}
                      style={{ width: '130px' }}
                      placeholder="Key Name"
                    />
                   <Input
                      addonBefore="Left"
                      value={item.left ?? ''}
                      onChange={(e) => handleLeftRightLeftChange(item.id, e.target.value)}
                       style={{ width: '130px' }}
                      placeholder="(null)"
                    />
                    <Input
                      addonBefore="Right"
                      value={item.right ?? ''}
                      onChange={(e) => handleLeftRightRightChange(item.id, e.target.value)}
                      style={{ width: '130px' }}
                      placeholder="(null)"
                    />
                  <Button
                    danger
                    icon={<MinusCircleOutlined />}
                    onClick={() => handleRemoveLeftRightMap(item.id)}
                  />
                </Space>
              </List.Item>
            )}
          />
          <Button
            type="dashed"
            onClick={handleAddLeftRightMap}
            block
            icon={<PlusOutlined />}
            style={{ marginTop: '10px' }}
          >
            Add Left Right Mapping
          </Button>
        </Panel>
      </Collapse>
    </Space>
  );
}; 
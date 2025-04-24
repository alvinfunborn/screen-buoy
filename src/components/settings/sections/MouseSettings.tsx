import React, { useState, useEffect } from 'react';
import { Form, InputNumber, List, Space, Spin, Typography, Select, Button } from 'antd';
import { Config, MouseStep, MouseConfig } from '@/types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import { useKeyOptions } from '../../../hooks/useKeyOptions';
const { Title, Text } = Typography;

interface MouseSettingsProps {
  loading?: boolean;
  onValuesChange?: (changedValues: any, allValues: Config) => void;
}

export const MouseSettings: React.FC<MouseSettingsProps> = ({ onValuesChange }) => {
  const form = Form.useFormInstance<Config>();
  const keyOptions = useKeyOptions(form);

  // --- Local State for Steps ---
  const [translateSteps, setTranslateSteps] = useState<MouseStep[]>([]);
  const [scrollSteps, setScrollSteps] = useState<MouseStep[]>([]);
  const [dragSteps, setDragSteps] = useState<MouseStep[]>([]);

  // --- Effect to Load Initial Data from Form ---
  useEffect(() => {
    const initialMouseConfig = form.getFieldValue('mouse') as MouseConfig;
    if (initialMouseConfig?.step) {
      setTranslateSteps(initialMouseConfig.step.translate || []);
      setScrollSteps(initialMouseConfig.step.scroll || []);
      setDragSteps(initialMouseConfig.step.drag || []);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [form]); // Depend on form instance

  // --- Function to Update Form and Notify Parent ---
  const syncFormAndNotify = (updatedSteps: { translate?: MouseStep[], scroll?: MouseStep[], drag?: MouseStep[] }) => {
    const currentMouseConfig = form.getFieldValue('mouse') as MouseConfig || {};
    const newStepConfig = { ...currentMouseConfig.step, ...updatedSteps }; // Merge updates
    const newMouseValue = {
      ...currentMouseConfig,
      step: newStepConfig
    };

    form.setFieldsValue({ mouse: newMouseValue });

    if (onValuesChange) {
        // Find which step type actually changed for more precise changedValues
        let changedKey: 'translate' | 'scroll' | 'drag' | null = null;
        if (updatedSteps.translate) changedKey = 'translate';
        else if (updatedSteps.scroll) changedKey = 'scroll';
        else if (updatedSteps.drag) changedKey = 'drag';

        if(changedKey) {
            const changedValues = { mouse: { step: { [changedKey]: newStepConfig[changedKey] } } };
            const allValues = form.getFieldsValue(true);
            onValuesChange(changedValues, allValues);
        } else { 
            // Fallback if somehow no key is identified (should not happen in current logic)
             const allValues = form.getFieldsValue(true);
             onValuesChange({ mouse: newMouseValue }, allValues);
        }
    }
  };

  // --- Handlers ---
  const updateStepConfig = (type: 'translate' | 'scroll' | 'drag', index: number, field: keyof MouseStep, value: any) => {
    let newSteps: MouseStep[] = [];
    let setSteps: React.Dispatch<React.SetStateAction<MouseStep[]>>;

    switch (type) {
      case 'translate':
        newSteps = [...translateSteps];
        setSteps = setTranslateSteps;
        break;
      case 'scroll':
        newSteps = [...scrollSteps];
        setSteps = setScrollSteps;
        break;
      case 'drag':
        newSteps = [...dragSteps];
        setSteps = setDragSteps;
        break;
    }

    if (!newSteps[index]) {
      newSteps[index] = { x: 0, y: 0, modifier: [] }; // Default structure if somehow missing
    }
    newSteps[index] = { ...newSteps[index], [field]: value };

    setSteps(newSteps); // Update local state first (triggers re-render)
    syncFormAndNotify({ [type]: newSteps }); // Sync with form and notify parent
  };

  const addStep = (type: 'translate' | 'scroll' | 'drag') => {
    let newSteps: MouseStep[] = [];
    let setSteps: React.Dispatch<React.SetStateAction<MouseStep[]>>;

     switch (type) {
      case 'translate':
        newSteps = [...translateSteps];
        setSteps = setTranslateSteps;
        break;
      case 'scroll':
        newSteps = [...scrollSteps];
        setSteps = setScrollSteps;
        break;
      case 'drag':
        newSteps = [...dragSteps];
        setSteps = setDragSteps;
        break;
    }

    newSteps.push({ x: 1, y: 1, modifier: [] }); // Add default new step

    setSteps(newSteps); // Update local state
    syncFormAndNotify({ [type]: newSteps }); // Sync and notify
  };

  const removeStep = (type: 'translate' | 'scroll' | 'drag', index: number) => {
     let newSteps: MouseStep[] = [];
     let setSteps: React.Dispatch<React.SetStateAction<MouseStep[]>>;

     switch (type) {
      case 'translate':
        newSteps = [...translateSteps];
        setSteps = setTranslateSteps;
        break;
      case 'scroll':
        newSteps = [...scrollSteps];
        setSteps = setScrollSteps;
        break;
      case 'drag':
        newSteps = [...dragSteps];
        setSteps = setDragSteps;
        break;
    }

    newSteps.splice(index, 1);

    setSteps(newSteps); // Update local state
    syncFormAndNotify({ [type]: newSteps }); // Sync and notify
  };

  const renderStepList = (type: 'translate' | 'scroll' | 'drag', label: string) => {
    // Determine the correct state array based on type
    let steps: MouseStep[];
    switch (type) {
        case 'translate': steps = translateSteps; break;
        case 'scroll': steps = scrollSteps; break;
        case 'drag': steps = dragSteps; break;
        default: steps = [];
    }

    return (
      <Form.Item label={label} style={{ width: '100%' }}>
        <Space direction="vertical" style={{ width: '100%' }}>
          <List
            bordered
            dataSource={steps}
            renderItem={(step, index) => (
              <List.Item>
                <Space align="center">
                  <InputNumber
                    addonBefore="X"
                    value={step.x}
                    min={-10000}
                    max={10000}
                    onChange={(value) => updateStepConfig(type, index, 'x', value)}
                    style={{ width: '80px' }}
                  />
                  <InputNumber
                    addonBefore="Y"
                    value={step.y}
                    min={-10000}
                    max={10000}
                    onChange={(value) => updateStepConfig(type, index, 'y', value)}
                    style={{ width: '80px' }}
                  />
                  <Text style={{ marginRight: 8 }}>Modifiers:</Text>
                  <Select
                    mode="multiple"
                    value={step.modifier}
                    options={keyOptions}
                    style={{ minWidth: '170px' }}
                    placeholder="Select modifiers"
                    onChange={(value) => updateStepConfig(type, index, 'modifier', value)}
                  />
                  <Button
                    danger
                    icon={<MinusCircleOutlined />}
                    onClick={() => removeStep(type, index)}
                  />
                </Space>
              </List.Item>
            )}
          />
          <Button
            type="dashed"
            onClick={() => addStep(type)}
            block
            icon={<PlusOutlined />}
          >
            Add {label}
          </Button>
        </Space>
      </Form.Item>
    );
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Mouse Settings</Title>
      
      {renderStepList('translate', 'Movement Step')}
      {renderStepList('scroll', 'Scroll Step')}
      {renderStepList('drag', 'Drag Step')}
      
    </Space>
  );
}; 
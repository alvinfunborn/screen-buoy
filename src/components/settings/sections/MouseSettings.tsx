import React from 'react';
import { Form, InputNumber, List, Space, Spin, Typography, Select, Button } from 'antd';
import { Config, MouseStep } from '@/types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';

const { Title } = Typography;

interface MouseSettingsProps {
  loading?: boolean;
}

export const MouseSettings: React.FC<MouseSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }
  
  const form = Form.useFormInstance<Config>();
  const values = form.getFieldsValue(true) as Config;
  const keyOptions = values?.keyboard?.available_key ? 
    Object.entries(values.keyboard.available_key).sort((a, b) => a[1] - b[1]).map(([key, _]) => ({
      label: key,
      value: key
    })) : [];
  keyOptions.push({
    label: "HintKey",
    value: "HintKey"
  });
  keyOptions.push({
    label: "HintRightKey",
    value: "HintRightKey"
  });
  keyOptions.push({
    label: "HintLeftKey",
    value: "HintLeftKey"
  });

  const updateStepConfig = (type: 'translate' | 'scroll' | 'drag', index: number, field: keyof MouseStep, value: any) => {
    const currentValues = form.getFieldsValue(true) as Config;
    const steps = [...(currentValues.mouse?.step?.[type] || [])];
    steps[index] = {
      ...steps[index],
      [field]: value
    };
    form.setFieldsValue({
      mouse: {
        ...currentValues.mouse,
        step: {
          ...currentValues.mouse?.step,
          [type]: steps
        }
      }
    });
  };

  const addStep = (type: 'translate' | 'scroll' | 'drag') => {
    const currentValues = form.getFieldsValue(true) as Config;
    const steps = [...(currentValues.mouse?.step?.[type] || [])];
    steps.push({ x: 1, y: 1, modifier: [] });
    form.setFieldsValue({
      mouse: {
        ...currentValues.mouse,
        step: {
          ...currentValues.mouse?.step,
          [type]: steps
        }
      }
    });
  };

  const removeStep = (type: 'translate' | 'scroll' | 'drag', index: number) => {
    const currentValues = form.getFieldsValue(true) as Config;
    const steps = [...(currentValues.mouse?.step?.[type] || [])];
    steps.splice(index, 1);
    form.setFieldsValue({
      mouse: {
        ...currentValues.mouse,
        step: {
          ...currentValues.mouse?.step,
          [type]: steps
        }
      }
    });
  };

  const renderStepList = (type: 'translate' | 'scroll' | 'drag', label: string) => {
    const steps = values?.mouse?.step?.[type] || [];
    
    return (
      <Form.Item label={label} style={{ width: '100%' }}>
        <Space direction="vertical" style={{ width: '100%' }}>
          <List
            bordered
            dataSource={steps}
            renderItem={(step, index) => (
              <List.Item>
                <Space>
                  <Form.Item label="X" style={{ marginBottom: 0 }}>
                    <InputNumber
                      value={step.x}
                      min={-10000}
                      max={10000}
                      onChange={(value) => updateStepConfig(type, index, 'x', value)}
                    />
                  </Form.Item>
                  <Form.Item label="Y" style={{ marginBottom: 0 }}>
                    <InputNumber
                      value={step.y}
                      min={-10000}
                      max={10000}
                      onChange={(value) => updateStepConfig(type, index, 'y', value)}
                    />
                  </Form.Item>
                  <Form.Item label="Modifiers" style={{ marginBottom: 0 }}>
                    <Select
                      mode="multiple"
                      value={step.modifier}
                      options={keyOptions}
                      style={{ width: '200px' }}
                      onChange={(value) => updateStepConfig(type, index, 'modifier', value)}
                    />
                  </Form.Item>
                  <MinusCircleOutlined onClick={() => removeStep(type, index)} />
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
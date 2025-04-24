import React from 'react';
import { Form, InputNumber, Space, Spin, Typography } from 'antd';

const { Title } = Typography;

interface MouseSettingsProps {
  loading?: boolean;
}

export const MouseSettings: React.FC<MouseSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Mouse Settings</Title>
      
      <Title level={4}>Movement Steps</Title>
      <Form.Item
        label="Translation Step X"
        name={['mouse', 'step', 'translate', 0, 'x']}
      >
        <InputNumber min={1} max={100} />
      </Form.Item>

      <Form.Item
        label="Translation Step Y"
        name={['mouse', 'step', 'translate', 0, 'y']}
      >
        <InputNumber min={1} max={100} />
      </Form.Item>

      <Title level={4}>Scroll Steps</Title>
      <Form.Item
        label="Scroll Step X"
        name={['mouse', 'step', 'scroll', 0, 'x']}
      >
        <InputNumber min={1} max={100} />
      </Form.Item>

      <Form.Item
        label="Scroll Step Y"
        name={['mouse', 'step', 'scroll', 0, 'y']}
      >
        <InputNumber min={1} max={100} />
      </Form.Item>

      <Title level={4}>Drag Steps</Title>
      <Form.Item
        label="Drag Step X"
        name={['mouse', 'step', 'drag', 0, 'x']}
      >
        <InputNumber min={1} max={100} />
      </Form.Item>

      <Form.Item
        label="Drag Step Y"
        name={['mouse', 'step', 'drag', 0, 'y']}
      >
        <InputNumber min={1} max={100} />
      </Form.Item>
    </Space>
  );
}; 
import React from 'react';
import { Form, Switch, InputNumber, Space, Typography, Spin, List, Input } from 'antd';
import { Config } from '@/types/config';

const { Title, Text, Paragraph } = Typography;

interface HintSettingsProps {
  loading?: boolean;
}

export const HintSettings: React.FC<HintSettingsProps> = ({ loading }) => {
  if (loading) {
    return <Spin />;
  }

  const form = Form.useFormInstance<Config>();
  // 直接从表单实例获取值
  const formValues = form.getFieldsValue(true) as Config;

  const hint_charsets = formValues?.hint?.charsets.map(item => item.join(', '))
  const hint_charset_extra = formValues?.hint?.charset_extra.join(', ')
  const hint_styles_default = JSON.stringify(formValues?.hint?.styles?.default)
  const hint_styles_types = formValues?.hint?.styles?.types.map(item => JSON.stringify(item))

  return (
    <Space direction="vertical" style={{ width: '100%' }}>
      <Title level={3}>Hint Settings</Title>
      
      <Form.Item
        label="Hint Charsets"
      >
        <List
          dataSource={hint_charsets}
          renderItem={(item) => 
            <List.Item>
              <Input value={item} />
            </List.Item>
          }
        />
      </Form.Item>

      <Form.Item
        label="Hint Charset Extra"
      >
        <Input value={hint_charset_extra} />
      </Form.Item>

      <Form.Item
        label="Hint Styles Default"
      >
        <Input.TextArea rows={4} value={hint_styles_default} />
      </Form.Item>
      
      <Form.Item
        label="Hint Styles Types"
      >
        <Paragraph style={{ fontSize: '11px' }}>
          type 0: hint for text
          <br/>type 1: hint for application window
          <br/>type 2: hint for application pane
          <br/>type 3: hint for Tab
          <br/>type 4: hint for Button
          <br/>type 5: hint for Scrollbar
        </Paragraph>
        <List
          dataSource={hint_styles_types}
          renderItem={(item) => 
            <List.Item>
              <Input.TextArea rows={2} value={item} />
            </List.Item>
          }    
        />
      </Form.Item>
    </Space>
  );
}; 
import React, { useState, useEffect } from 'react';
import { Form, InputNumber, List, Space, Typography, Select, Button, Tooltip } from 'antd';
import { Config, MouseStep, MouseConfig } from '@/types/config';
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';
import { useKeyOptions } from '../../../hooks/useKeyOptions';
import { SettingSection } from '../SettingSection';
import { useTranslation } from 'react-i18next';

const { Paragraph } = Typography;

interface MouseSettingsProps {
  onValuesChange?: (changedValues: any, allValues: Config) => void;
  availableKeysData?: Record<string, number>;
}

export const MouseSettings: React.FC<MouseSettingsProps> = ({ onValuesChange, availableKeysData }) => {
  const { t } = useTranslation();
  const form = Form.useFormInstance<Config>();
  const keyOptions = useKeyOptions(availableKeysData);

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

  // --- Function to Update Form and Notify ---
  const syncFormAndNotify = (updatedSteps: { translate?: MouseStep[], scroll?: MouseStep[], drag?: MouseStep[] }) => {
    const currentMouseConfig = form.getFieldValue('mouse') as MouseConfig || {};
    const currentStepConfig = currentMouseConfig.step || {
      translate: [],
      scroll: [],
      drag: []
    };

    // 确保所有必需的字段都存在
    const newStepConfig = {
      translate: currentStepConfig.translate || [],
      scroll: currentStepConfig.scroll || [],
      drag: currentStepConfig.drag || [],
      ...updatedSteps
    };

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

      if (changedKey) {
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

    // 确保 modifier 字段存在
    if (field === 'modifier' && value === undefined) {
      value = [];
    }

    newSteps[index] = {
      ...newSteps[index],
      [field]: value,
      modifier: field === 'modifier' ? value : (newSteps[index].modifier || [])
    };

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

    newSteps.push({
      x: 1,
      y: 1,
      modifier: []
    }); // Add default new step with empty modifier array

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
    let tooltip = '';
    switch (type) {
      case 'translate': steps = translateSteps; tooltip = t('mouse.movementStepTooltip'); break;
      case 'scroll': steps = scrollSteps; tooltip = t('mouse.scrollStepTooltip'); break;
      case 'drag': steps = dragSteps; tooltip = t('mouse.dragStepTooltip'); break;
      default: steps = []; tooltip = '';
    }

    return (
      <SettingSection title={t(label)} description={tooltip}>
        <Space direction="vertical" style={{ width: '100%' }}>
          <List
            bordered
            dataSource={steps}
            renderItem={(step, index) => (
              <List.Item>
                <div className="mouse-step-row">
                  <InputNumber
                    addonBefore={t('mouse.x')}
                    aria-label={`${t(label)} ${index + 1} X`}
                    value={step.x}
                    min={-10000}
                    max={10000}
                    onChange={(value) => updateStepConfig(type, index, 'x', value)}
                    style={{ width: '100%' }}
                  />
                  <InputNumber
                    addonBefore={t('mouse.y')}
                    aria-label={`${t(label)} ${index + 1} Y`}
                    value={step.y}
                    min={-10000}
                    max={10000}
                    onChange={(value) => updateStepConfig(type, index, 'y', value)}
                    style={{ width: '100%' }}
                  />
                  <Tooltip title={t('mouse.modifiersTooltip')}>
                    <Paragraph style={{ fontWeight: 'normal', marginBottom: 0 }}>
                      {t('mouse.modifiers')}
                    </Paragraph>
                  </Tooltip>
                  <Select
                    mode="multiple"
                    aria-label={`${t('mouse.modifiers')} ${index + 1}`}
                    value={step.modifier}
                    options={keyOptions}
                    style={{ width: '100%' }}
                    placeholder={t('mouse.selectModifiers')}
                    onChange={(value) => updateStepConfig(type, index, 'modifier', value)}
                  />
                  <Button
                    danger
                    type="text"
                    aria-label={t('settings.removeItem', { name: `${t(label)} ${index + 1}` })}
                    icon={<MinusCircleOutlined />}
                    onClick={() => removeStep(type, index)}
                  />
                </div>
              </List.Item>
            )}
          />
          <Button
            type="dashed"
            onClick={() => addStep(type)}
            block
            icon={<PlusOutlined />}>
            {t('mouse.addStep', { label: t(label) })}
          </Button>
        </Space>
      </SettingSection>
    );
  };

  return (
    <div className="settings-stack">
      {renderStepList('translate', 'mouse.movementStep')}
      {renderStepList('scroll', 'mouse.scrollStep')}
      {renderStepList('drag', 'mouse.dragStep')}
    </div>
  );
}; 
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MouseConfig {
    pub step: MouseStepConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MouseStepConfig {
    pub translate: Vec<MouseStep>,
    pub scroll: Vec<MouseStep>,
    pub drag: Vec<MouseStep>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MouseStep {
    pub x: i32,
    pub y: i32,
    pub modifier: Option<Vec<String>>,
}

impl MouseConfig {
    pub fn get_translate_step(&self, hold_modifier: &Vec<String>) -> &MouseStep {
        let mut unmodified_step = &self.step.translate[0];
        for m in self.step.translate.iter() {
            if let Some(modifier_config) = &m.modifier {
                for modifier in modifier_config.iter() {
                    if hold_modifier.contains(modifier) {
                        return m;
                    }
                }
            } else {
                unmodified_step = m;
            }
        }
        unmodified_step
    }

    pub fn get_scroll_step(&self, hold_modifier: &Vec<String>) -> &MouseStep {
        let mut unmodified_step = &self.step.scroll[0];
        for m in self.step.scroll.iter() {
            if let Some(modifier_config) = &m.modifier {
                for modifier in modifier_config.iter() {
                    if hold_modifier.contains(modifier) {
                        return m;
                    }
                }
            } else {
                unmodified_step = m;
            }
        }
        unmodified_step
    }

    pub fn get_drag_step(&self, hold_modifier: &Vec<String>) -> &MouseStep {
        let mut unmodified_step = &self.step.drag[0];
        for m in self.step.drag.iter() {
            if let Some(modifier_config) = &m.modifier {
                for modifier in modifier_config.iter() {
                    if hold_modifier.contains(modifier) {
                        return m;
                    }
                }
            } else {
                unmodified_step = m;
            }
        }
        unmodified_step
    }

    pub fn get_modifiers(&self) -> HashSet<String> {
        let mut modifiers: HashSet<String> = HashSet::new();
        for m in self.step.translate.iter() {
            if let Some(modifier_config) = &m.modifier {
                for modifier in modifier_config.iter() {
                    modifiers.insert(modifier.to_string());
                }
            }
        }
        modifiers
    }
}

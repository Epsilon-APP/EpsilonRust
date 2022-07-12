use crate::epsilon::server::instances::common::instance_type::InstanceType;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub struct Label {
    key: String,
    value: String,
}

impl Label {
    pub const DEFAULT_LABEL: &'static str = "epsilon.fr/default";

    pub const INSTANCE_TYPE_LABEL: &'static str = "epsilon.fr/instance";
    pub const TEMPLATE_LABEL: &'static str = "epsilon.fr/template";
    pub const SLOTS_LABEL: &'static str = "epsilon.fr/slots";
    pub const IN_GAME_LABEL: &'static str = "epsilon.fr/in-game";

    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: String::from(key),
            value: String::from(value),
        }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn concat(labels: &[Self]) -> String {
        let mut str = String::new();

        for label in labels {
            str.push_str(&label.to_string());
            str.push(',');
        }

        if !str.is_empty() {
            str.pop();
        }

        str
    }

    pub fn get_default_label() -> Label {
        Label::new(Self::DEFAULT_LABEL, "true")
    }

    pub fn get_instance_type_label(instance_type: &InstanceType) -> Label {
        Label::new(Self::INSTANCE_TYPE_LABEL, &instance_type.to_string())
    }

    pub fn get_template_label(template_name: &str) -> Label {
        Label::new(Self::TEMPLATE_LABEL, template_name)
    }

    pub fn get_slots_label(slots: i32) -> Label {
        Label::new(Self::SLOTS_LABEL, slots.to_string().as_str())
    }

    pub fn get_in_game_label(enable: bool) -> Label {
        let str = if enable { "true" } else { "false" };

        Label::new(Self::IN_GAME_LABEL, str)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", &self.key, &self.value)
    }
}

use crate::epsilon::server::instance_type::InstanceType;
use std::fmt::{Display, Formatter};

pub struct Label {
    key: String,
    value: String,
}

impl Label {
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

    pub fn get_instance_type_label(instance_type: &InstanceType) -> Label {
        Label::new("epsilon.fr/instance", &instance_type.to_string())
    }

    pub fn get_template_label(template_name: &str) -> Label {
        Label::new("epsilon.fr/template", template_name)
    }

    pub fn get_slots_label(slots: i32) -> Label {
        Label::new("epsilon.fr/slots", slots.to_string().as_str())
    }

    pub fn get_in_game_label() -> Label {
        Label::new("epsilon.fr/in-game", "true")
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", &self.key, &self.value)
    }
}

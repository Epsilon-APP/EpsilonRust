use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::k8s::label::Label;
use futures::StreamExt;
use rocket::http::ext::IntoCollection;
use std::collections::BTreeMap;

pub struct LabelsBuilder {
    labels: Vec<Label>,
}

impl LabelsBuilder {
    pub fn new() -> LabelsBuilder {
        let default = Label::new(Label::DEFAULT_LABEL, "true");

        LabelsBuilder {
            labels: vec![default],
        }
    }

    pub fn with_instance_type_label(&mut self, instance_type: &InstanceType) -> &Self {
        self.labels.push(Label::new(
            Label::INSTANCE_TYPE_LABEL,
            &instance_type.to_string(),
        ));

        &self
    }

    pub fn with_template_label(&mut self, template_name: &str) -> &Self {
        self.labels
            .push(Label::new(Label::TEMPLATE_LABEL, template_name));

        &self
    }

    pub fn with_slots_label(&mut self, slots: i32) -> &Self {
        self.labels
            .push(Label::new(Label::SLOTS_LABEL, &slots.to_string()));

        &self
    }

    pub fn with_in_game_label(&mut self, enable: bool) -> &Self {
        self.labels.push(Label::new(
            Label::IN_GAME_LABEL,
            if enable { "true" } else { "false" },
        ));

        &self
    }

    pub fn build(&self) -> BTreeMap<String, String> {
        self.labels
            .into_iter()
            .map(|label| {
                (
                    String::from(label.get_key()),
                    String::from(label.get_value()),
                )
            })
            .collect()
    }
}

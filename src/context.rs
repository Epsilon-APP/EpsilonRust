use crate::{EpsilonApi, InstanceProvider, QueueProvider, TemplateProvider};
use std::sync::Arc;

pub struct Context {
    epsilon_api: EpsilonApi,
    template_provider: Arc<TemplateProvider>,
    instance_provider: InstanceProvider,
    queue_provider: QueueProvider,
}

impl Context {
    pub fn new(
        epsilon_api: EpsilonApi,
        template_provider: Arc<TemplateProvider>,
        instance_provider: InstanceProvider,
        queue_provider: QueueProvider,
    ) -> Arc<Context> {
        Arc::new(Self {
            epsilon_api,
            template_provider,
            instance_provider,
            queue_provider,
        })
    }

    pub fn get_epsilon_api(&self) -> &EpsilonApi {
        &self.epsilon_api
    }

    pub fn get_template_provider(&self) -> &Arc<TemplateProvider> {
        &self.template_provider
    }

    pub fn get_instance_provider(&self) -> &InstanceProvider {
        &self.instance_provider
    }

    pub fn get_queue_provider(&self) -> &QueueProvider {
        &self.queue_provider
    }
}

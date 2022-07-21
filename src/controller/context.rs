use crate::controller::definitions::epsilon_instance::EpsilonInstance;
use crate::TemplateProvider;
use k8s_openapi::api::core::v1::Pod;
use kube::Api;
use std::sync::Arc;

pub struct Context {
    pub pod_api: Api<Pod>,
    pub epsilon_instance_api: Api<EpsilonInstance>,
    pub template_provider: Arc<TemplateProvider>,
}

impl Context {
    pub fn new(
        pod_api: Api<Pod>,
        epsilon_instance_api: Api<EpsilonInstance>,
        template_provider: &Arc<TemplateProvider>,
    ) -> Self {
        Context {
            pod_api,
            epsilon_instance_api,
            template_provider: Arc::clone(template_provider),
        }
    }
}

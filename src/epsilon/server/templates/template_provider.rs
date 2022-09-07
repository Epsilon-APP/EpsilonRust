use std::env;
use std::sync::Arc;

use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::server::templates::template::Template;
use crate::EpsilonConfig;

pub struct TemplateProvider {
    config: Arc<EpsilonConfig>,
}

impl TemplateProvider {
    pub fn new(config: &Arc<EpsilonConfig>) -> Arc<TemplateProvider> {
        Arc::new(Self {
            config: Arc::clone(config),
        })
    }

    #[inline]
    pub async fn get_proxy_template(&self) -> Result<Template, EpsilonError> {
        let proxy_template = &self.config.proxy.template;

        self.get_template(proxy_template).await
    }

    #[inline]
    pub async fn get_hub_template(&self) -> Result<Template, EpsilonError> {
        let hub_template = &self.config.hub.template;

        self.get_template(hub_template).await
    }

    #[inline]
    pub fn is_proxy(&self, template: &Template) -> bool {
        template.name == self.config.proxy.template
    }

    #[inline]
    pub fn is_hub(&self, template: &Template) -> bool {
        template.name == self.config.hub.template
    }

    pub async fn get_template(&self, template_name: &str) -> Result<Template, EpsilonError> {
        let url = self.get_template_host(&format!("templates/{}", template_name));

        debug!("Fetching template from {}", url);

        let request = reqwest::get(&url).await?;

        Ok(request
            .json::<Template>()
            .await
            .map_err(|_| EpsilonError::ParseJsonError("Get Template".to_owned()))?
        )
    }

    pub async fn get_templates(&self) -> Result<Vec<Template>, EpsilonError> {
        let url = self.get_template_host("templates");

        debug!("Fetching template list from {}", url);

        let request = reqwest::get(&url).await?;

        Ok(request
            .json::<Vec<Template>>()
            .await
            .map_err(|_| EpsilonError::ParseJsonError("Get Templates".to_owned()))?
        )
    }

    #[inline]
    fn get_template_host(&self, route: &str) -> String {
        format!(
            "http://{}:8000/{}",
            env::var("HOST_TEMPLATE").expect("Failed to get HOST_TEMPLATE Environment"),
            route
        )
    }
}

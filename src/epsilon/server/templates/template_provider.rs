use crate::epsilon::server::templates::template::Template;
use crate::{EResult, EpsilonConfig};
use anyhow::format_err;
use std::env;
use std::sync::Arc;

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
    pub async fn get_proxy_template(&self) -> EResult<Template> {
        let proxy_template = &self.config.proxy.template;

        self.get_template(proxy_template).await
    }

    #[inline]
    pub async fn get_hub_template(&self) -> EResult<Template> {
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

    pub async fn get_template(&self, template_name: &str) -> EResult<Template> {
        let url = self.get_template_host(&format!("templates/{}", template_name));

        debug!("Fetching template from {}", url);

        let request = reqwest::get(&url).await?;

        if request.status().is_success() {
            let template = request.json::<Template>().await?;

            Ok(template)
        } else {
            Err(format_err!("Failed to fetch template from {}", url))
        }
    }

    pub async fn get_templates(&self) -> EResult<Vec<Template>> {
        let url = self.get_template_host("templates");

        debug!("Fetching template list from {}", url);

        let request = reqwest::get(&url).await?;

        if request.status().is_success() {
            let template = request.json::<Vec<Template>>().await?;

            Ok(template)
        } else {
            Err(format_err!("Failed to fetch template list from {}", url))
        }
    }

    #[inline]
    fn get_template_host(&self, route: &str) -> String {
        format!(
            "http://{}:80/{}",
            env::var("HOST_TEMPLATE").unwrap_or(String::from("dev-template.epsilon-srv.me")),
            route
        )
    }
}

use crate::epsilon::server::templates::template::Template;
use crate::EResult;
use anyhow::format_err;
use std::env;

struct TemplateProvider;

impl TemplateProvider {
    pub fn new() -> TemplateProvider {
        TemplateProvider
    }

    pub async fn get_template_host(&self, route: &str) -> String {
        format!(
            "http://{}:8000/{}",
            env::var("HOST_TEMPLATE").unwrap(),
            route
        )
    }

    pub async fn get_template(&self, template_name: &str) -> EResult<Template> {
        let url = self
            .get_template_host(&format!("templates/{}", template_name))
            .await;

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
        let url = self.get_template_host("templates").await;

        debug!("Fetching template list from {}", url);

        let request = reqwest::get(&url).await?;

        if request.status().is_success() {
            let template = request.json::<Vec<Template>>().await?;

            Ok(template)
        } else {
            Err(format_err!("Failed to fetch template list from {}", url))
        }
    }
}

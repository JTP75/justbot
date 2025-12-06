use crate::app::{puetce::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct SetModelCommand;

impl Command for SetModelCommand {
    fn name(&self) -> &str { "set-model" }
    fn aliases(&self) -> Vec<&str> { vec!["setm"] }
    fn desc(&self) -> &str { "Set the currently selected model. The default model will be used otherwise." }
    fn help(&self) -> &str { "Usage: set-model [<model_id> | <model_id_number>]\n\t- `model_id` must be a valid anthropic model ID\n\t- `model_id_number` is an integer [0,1,2] for haiku, sonnet, and opus respectively\nLeaving the field blank will set the current model to the configured default" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let model_id = args
            .first()
            .map(|arg| {
                arg.parse::<usize>()
                    .ok()
                    .and_then(|num| crate::common::config::MODELS.get(num))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| arg.clone())
            })
            .map_or_else(
                || crate::common::config
                    ::get_config::<String>("anthropic_config.json", "default_model"),
                Ok,
            )?;
        bot.set_model(&model_id);
        Ok(Some(format!("Model set to '{model_id}'")))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "set-model".into(), 
        || Box::new(SetModelCommand),
    );
}
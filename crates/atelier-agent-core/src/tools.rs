//! Registration/argument checks adapted from ShellPilot ToolRegistry.
//! A validated call carries data only: it is never an execution permit.
use crate::{schema::Schema, Error, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, (ToolDefinition, Schema)>,
}

pub struct ValidatedArguments {
    name: String,
    arguments: Value,
}
impl ValidatedArguments {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn arguments(&self) -> &Value {
        &self.arguments
    }
}
impl ToolRegistry {
    pub fn register(&mut self, definition: ToolDefinition) -> Result<()> {
        let valid_name = !definition.name.is_empty()
            && definition.name.len() <= 64
            && definition
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'));
        if !valid_name
            || self.tools.len() >= 64
            || self.tools.contains_key(&definition.name)
            || definition.description.len() > 16000
            || definition.parameters.to_string().len() > 65536
        {
            return Err(Error::Invalid("invalid tool definition".into()));
        }
        let schema = Schema::compile(&definition.parameters)
            .ok_or_else(|| Error::Invalid("unsupported tool schema".into()))?;
        self.tools
            .insert(definition.name.clone(), (definition, schema));
        Ok(())
    }
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|(definition, _)| definition.clone())
            .collect()
    }
    pub fn validate(&self, name: &str, raw: &str) -> Result<ValidatedArguments> {
        let (_, schema) = self
            .tools
            .get(name)
            .ok_or_else(|| Error::Invalid("tool is not registered".into()))?;
        let arguments = schema
            .parse(raw, |_| {})
            .ok_or_else(|| Error::Invalid("invalid tool arguments".into()))?;
        Ok(ValidatedArguments {
            name: name.into(),
            arguments,
        })
    }
}

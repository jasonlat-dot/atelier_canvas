use atelier_agent_core::{
    budget::{estimate_text, HistoryGroup, InputCosts, ModelBudget},
    schema::Schema,
    strict_json,
    tools::{ToolDefinition, ToolRegistry},
    Error,
};
use serde_json::json;

fn budget() -> ModelBudget {
    ModelBudget {
        context_window: 1000,
        output_reserve: 100,
        safety_reserve: 100,
        compact_percent: 75,
    }
}
fn group(ids: &[u64], tokens: u64) -> HistoryGroup {
    HistoryGroup {
        event_ids: ids.into(),
        tokens,
        complete: true,
    }
}
fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_text_node".into(),
        description: "Create a text node".into(),
        parameters: json!({"type":"object","properties":{
            "text":{"type":"string","minLength":1,"maxLength":100},
            "count":{"type":"integer","minimum":1,"maximum":4}},
            "required":["text","count"],"additionalProperties":false}),
    }
}
fn registry() -> ToolRegistry {
    let mut tools = ToolRegistry::default();
    tools.register(definition()).unwrap();
    tools
}

#[test]
fn strict_json_rejects_duplicates_at_any_depth() {
    for raw in [r#"{"x":1,"x":2}"#, r#"{"items":[{"x":1,"x":2}]}"#] {
        assert!(strict_json::parse(raw).is_err());
        assert!(strict_json::parse_slice(raw.as_bytes()).is_err());
    }
    assert!(strict_json::parse(r#"{"text":"创作","items":[1,true,null]}"#).is_ok());
}
#[test]
fn strict_json_rejects_malformed_and_trailing_input() {
    for raw in ["{", "{}{}", "NaN", r#"{"x":Infinity}"#] {
        assert!(strict_json::parse(raw).is_err());
    }
}
#[test]
fn only_registered_tools_accept_validated_data() {
    let tools = registry();
    let call = tools
        .validate("create_text_node", r#"{"text":"品牌海报","count":2}"#)
        .unwrap();
    assert_eq!(call.name(), "create_text_node");
    assert_eq!(call.arguments()["count"], 2);
    assert!(tools.validate("ssh_execute", "{}").is_err());
    assert_eq!(tools.definitions().len(), 1);
}
#[test]
fn model_arguments_cannot_add_permissions_or_erase_duplicates() {
    let tools = registry();
    for raw in [
        r#"{"text":"x","count":1,"approved":true}"#,
        r#"{"text":"x","count":1,"count":4}"#,
        r#"{"count":1}"#,
        r#"{"text":"x","count":1.0}"#,
        r#"{"text":"x","count":0}"#,
        r#"{"text":"x","count":5}"#,
        r#"{"text":"","count":1}"#,
    ] {
        assert!(tools.validate("create_text_node", raw).is_err(), "{raw}");
    }
}
#[test]
fn schemas_cannot_silently_enable_unsupported_constraints() {
    for schema in [
        json!({"type":"object","properties":{"x":{"$ref":"remote"}}}),
        json!({"type":"object","properties":{},"required":[],"additionalProperties":true}),
        json!({"type":"object","properties":{},"required":[],"additionalProperties":false,"anyOf":[]}),
    ] {
        assert!(Schema::compile(&schema).is_none());
    }
}
#[test]
fn nested_values_enums_and_array_limits_are_checked() {
    let schema=Schema::compile(&json!({"type":"object","properties":{
        "refs":{"type":"array","minItems":1,"maxItems":2,"items":{"type":"string","enum":["a","b"]}}},
        "required":["refs"],"additionalProperties":false})).unwrap();
    assert!(schema.parse(r#"{"refs":["a","b"]}"#, |_| {}).is_some());
    for raw in [
        r#"{"refs":[]}"#,
        r#"{"refs":["c"]}"#,
        r#"{"refs":["a","a","a"]}"#,
    ] {
        assert!(schema.parse(raw, |_| {}).is_none());
    }
}
#[test]
fn oversized_arguments_are_rejected_before_parsing() {
    assert!(registry()
        .validate("create_text_node", &" ".repeat(65537))
        .is_err());
}
#[test]
fn registry_rejects_duplicates_and_invalid_names() {
    let mut tools = registry();
    assert!(tools.register(definition()).is_err());
    let mut invalid = definition();
    invalid.name = "unsafe.name".into();
    assert!(tools.register(invalid).is_err());
}
#[test]
fn budget_includes_all_mandatory_input_categories() {
    let plan = budget()
        .plan(
            &InputCosts {
                fixed_tokens: 100,
                pinned_tokens: 200,
                media_tokens: 50,
            },
            &[],
        )
        .unwrap();
    assert_eq!(plan.available_input_tokens, 800);
    assert_eq!(plan.estimated_input_tokens, 350);
    assert!(!plan.should_compact);
}
#[test]
fn oversized_mandatory_input_is_not_silently_dropped() {
    assert!(matches!(
        budget().plan(
            &InputCosts {
                pinned_tokens: 801,
                ..Default::default()
            },
            &[]
        ),
        Err(Error::BudgetExceeded)
    ));
}
#[test]
fn unresolved_tool_groups_block_a_model_request() {
    let mut unresolved = group(&[1, 2], 100);
    unresolved.complete = false;
    assert!(matches!(
        budget().plan(&InputCosts::default(), &[unresolved]),
        Err(Error::ToolProtocol)
    ));
}
#[test]
fn history_groups_are_kept_atomically_and_chronologically() {
    let plan = budget()
        .plan(
            &InputCosts {
                pinned_tokens: 100,
                ..Default::default()
            },
            &[
                group(&[1], 300),
                group(&[2, 3, 4, 5], 400),
                group(&[6], 100),
            ],
        )
        .unwrap();
    assert_eq!(plan.selected_event_ids, vec![2, 3, 4, 5, 6]);
    assert_eq!(plan.dropped_groups, 1);
    assert_eq!(plan.estimated_full_input_tokens, 900);
    assert!(plan.should_compact);
}
#[test]
fn budget_signals_compaction_before_overflow() {
    let plan = budget()
        .plan(&InputCosts::default(), &[group(&[1], 600)])
        .unwrap();
    assert!(plan.should_compact);
    assert_eq!(plan.dropped_groups, 0);
}
#[test]
fn invalid_budget_and_arithmetic_overflow_fail_closed() {
    let mut invalid = budget();
    invalid.compact_percent = 100;
    assert!(invalid.available_input().is_err());
    invalid = budget();
    invalid.output_reserve = 0;
    assert!(invalid.available_input().is_err());
    invalid = budget();
    invalid.safety_reserve = 1000;
    assert!(invalid.available_input().is_err());
    assert!(matches!(
        budget().plan(
            &InputCosts {
                fixed_tokens: u64::MAX,
                pinned_tokens: 1,
                media_tokens: 0
            },
            &[]
        ),
        Err(Error::BudgetExceeded)
    ));
}
#[test]
fn invalid_history_group_identity_is_rejected() {
    for groups in [
        vec![group(&[], 10)],
        vec![group(&[0], 10)],
        vec![group(&[1], 0)],
        vec![group(&[1], 10), group(&[1], 10)],
    ] {
        assert!(budget().plan(&InputCosts::default(), &groups).is_err());
    }
}
#[test]
fn unicode_estimate_is_explicitly_a_byte_heuristic() {
    assert_eq!(estimate_text(""), 0);
    assert_eq!(estimate_text("中文"), 2);
    assert_eq!(estimate_text("abcd"), 2);
}

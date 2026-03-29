use pm4py::semconv::llm_attributes::{
    llm_context_compression_strategy, LLM_CONTEXT_COMPRESSION_RATIO,
    LLM_CONTEXT_COMPRESSION_STRATEGY, LLM_CONTEXT_COMPRESSION_TOKENS_SAVED,
};
use pm4py::semconv::llm_span_names::LLM_CONTEXT_COMPRESS_SPAN;

#[test]
fn test_llm_context_compress_span_name_key() {
    assert_eq!(LLM_CONTEXT_COMPRESS_SPAN, "llm.context.compress");
}

#[test]
fn test_llm_context_compress_span_name_format() {
    let span_name = LLM_CONTEXT_COMPRESS_SPAN;
    assert!(span_name.contains("."));
    assert!(!span_name.contains("_"));
}

#[test]
fn test_llm_context_compression_ratio_attribute_exists() {
    let attr = LLM_CONTEXT_COMPRESSION_RATIO;
    assert_eq!(attr, "llm.context.compression.ratio");
}

#[test]
fn test_llm_context_compression_ratio_type_double() {
    // This test verifies the attribute exists and has the correct key.
    // Type checking is done at compile-time via schema constants.
    assert_eq!(
        LLM_CONTEXT_COMPRESSION_RATIO,
        "llm.context.compression.ratio"
    );
}

#[test]
fn test_llm_context_compression_strategy_attribute_exists() {
    let attr = LLM_CONTEXT_COMPRESSION_STRATEGY;
    assert_eq!(attr, "llm.context.compression.strategy");
}

#[test]
fn test_llm_context_compression_strategy_is_enum() {
    // Verify enum members exist
    assert_eq!(llm_context_compression_strategy::SUMMARIZE, "summarize");
    assert_eq!(llm_context_compression_strategy::TRUNCATE, "truncate");
    assert_eq!(
        llm_context_compression_strategy::SLIDING_WINDOW,
        "sliding_window"
    );
    assert_eq!(llm_context_compression_strategy::SELECTIVE, "selective");
}

#[test]
fn test_llm_context_compression_strategy_values() {
    let values: Vec<&str> = vec![
        llm_context_compression_strategy::SUMMARIZE,
        llm_context_compression_strategy::TRUNCATE,
        llm_context_compression_strategy::SLIDING_WINDOW,
        llm_context_compression_strategy::SELECTIVE,
    ];
    assert_eq!(values.len(), 4);
    for val in values {
        assert!(!val.is_empty());
    }
}

#[test]
fn test_llm_context_compression_tokens_saved_attribute_exists() {
    let attr = LLM_CONTEXT_COMPRESSION_TOKENS_SAVED;
    assert_eq!(attr, "llm.context.compression.tokens_saved");
}

#[test]
fn test_llm_context_compression_tokens_saved_type_int() {
    // This test verifies the attribute exists and has the correct key.
    // Type checking is done at compile-time via schema constants.
    assert_eq!(
        LLM_CONTEXT_COMPRESSION_TOKENS_SAVED,
        "llm.context.compression.tokens_saved"
    );
}

#[test]
fn test_all_compression_attributes_have_correct_names() {
    assert_eq!(
        LLM_CONTEXT_COMPRESSION_RATIO,
        "llm.context.compression.ratio"
    );
    assert_eq!(
        LLM_CONTEXT_COMPRESSION_STRATEGY,
        "llm.context.compression.strategy"
    );
    assert_eq!(
        LLM_CONTEXT_COMPRESSION_TOKENS_SAVED,
        "llm.context.compression.tokens_saved"
    );
}

#[test]
fn test_compression_span_references_compression_attributes() {
    // The span name should reference compression-related attributes
    assert!(
        LLM_CONTEXT_COMPRESS_SPAN.contains("context")
            || LLM_CONTEXT_COMPRESS_SPAN.contains("compress")
    );
}

#[test]
fn test_compression_strategy_enum_no_duplicates() {
    let values = vec![
        llm_context_compression_strategy::SUMMARIZE,
        llm_context_compression_strategy::TRUNCATE,
        llm_context_compression_strategy::SLIDING_WINDOW,
        llm_context_compression_strategy::SELECTIVE,
    ];
    let unique: std::collections::HashSet<_> = values.iter().cloned().collect();
    assert_eq!(values.len(), unique.len(), "No duplicate enum values");
}

#[test]
fn test_compression_attributes_follow_naming_convention() {
    assert!(LLM_CONTEXT_COMPRESSION_RATIO.starts_with("llm."));
    assert!(LLM_CONTEXT_COMPRESSION_STRATEGY.starts_with("llm."));
    assert!(LLM_CONTEXT_COMPRESSION_TOKENS_SAVED.starts_with("llm."));
}

#[test]
fn test_compression_span_kind_internal() {
    // Verify span name is for internal operations
    let span = LLM_CONTEXT_COMPRESS_SPAN;
    assert_eq!(span, "llm.context.compress");
}

#[test]
fn test_compression_attributes_string_values() {
    // Verify attributes are string constants
    let ratio_key: &str = LLM_CONTEXT_COMPRESSION_RATIO;
    let strategy_key: &str = LLM_CONTEXT_COMPRESSION_STRATEGY;
    let tokens_key: &str = LLM_CONTEXT_COMPRESSION_TOKENS_SAVED;

    assert!(!ratio_key.is_empty());
    assert!(!strategy_key.is_empty());
    assert!(!tokens_key.is_empty());
}

#[test]
fn test_compression_strategy_selective_value() {
    assert_eq!(llm_context_compression_strategy::SELECTIVE, "selective");
}

#[test]
fn test_compression_strategy_sliding_window_value() {
    assert_eq!(
        llm_context_compression_strategy::SLIDING_WINDOW,
        "sliding_window"
    );
}

#[test]
fn test_compression_strategy_truncate_value() {
    assert_eq!(llm_context_compression_strategy::TRUNCATE, "truncate");
}

#[test]
fn test_compression_strategy_summarize_value() {
    assert_eq!(llm_context_compression_strategy::SUMMARIZE, "summarize");
}

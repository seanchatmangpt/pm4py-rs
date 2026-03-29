/// LLM adapter application — applying a parameter-efficient fine-tuning adapter to customize a base model.
///
/// Span: `span.llm.adapter.apply`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_ADAPTER_APPLY_SPAN: &str = "llm.adapter.apply";
/// LLM batch inference job — processing multiple requests in a single batch for efficiency.
///
/// Span: `span.llm.batch.run`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_BATCH_RUN_SPAN: &str = "llm.batch.run";
/// LLM response cache lookup — checks if a cached response exists for the given prompt hash.
///
/// Span: `span.llm.cache.lookup`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_CACHE_LOOKUP_SPAN: &str = "llm.cache.lookup";
/// Executing chain-of-thought reasoning — multi-step LLM inference with intermediate reasoning.
///
/// Span: `span.llm.chain_of_thought`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_CHAIN_OF_THOUGHT_SPAN: &str = "llm.chain_of_thought";
/// Context compression — reducing token count of context using configured strategy.
///
/// Span: `span.llm.context.compress`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_CONTEXT_COMPRESS_SPAN: &str = "llm.context.compress";
/// Processing a single context compression operation — validates compression ratio and token savings.
///
/// Span: `span.llm.context.compress.process`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_CONTEXT_COMPRESS_PROCESS_SPAN: &str = "llm.context.compress.process";
/// Context window management — handles overflow by applying the configured strategy.
///
/// Span: `span.llm.context.manage`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_CONTEXT_MANAGE_SPAN: &str = "llm.context.manage";
/// Recording cost for a completed LLM inference — captures input/output token costs.
///
/// Span: `span.llm.cost.record`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_COST_RECORD_SPAN: &str = "llm.cost.record";
/// Knowledge distillation training — transferring knowledge from teacher to student model.
///
/// Span: `span.llm.distillation.train`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_DISTILLATION_TRAIN_SPAN: &str = "llm.distillation.train";
/// LLM embedding generation — converting text input into dense vector representations for semantic search or retrieval.
///
/// Span: `span.llm.embedding.generate`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_EMBEDDING_GENERATE_SPAN: &str = "llm.embedding.generate";
/// Evaluating an LLM response quality using a scoring rubric.
///
/// Span: `span.llm.evaluation`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_EVALUATION_SPAN: &str = "llm.evaluation";
/// Few-shot example retrieval — selecting and ranking examples for in-context learning.
///
/// Span: `span.llm.few_shot.retrieve`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_FEW_SHOT_RETRIEVE_SPAN: &str = "llm.few_shot.retrieve";
/// LLM fine-tuning job execution — training a language model on domain-specific data.
///
/// Span: `span.llm.finetune.run`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_FINETUNE_RUN_SPAN: &str = "llm.finetune.run";
/// LLM function call routing — directing a function call from LLM output to the appropriate handler.
///
/// Span: `span.llm.function_call.route`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_FUNCTION_CALL_ROUTE_SPAN: &str = "llm.function_call.route";
/// Evaluating LLM safety guardrails on a request or response.
///
/// Span: `span.llm.guardrail.check`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_GUARDRAIL_CHECK_SPAN: &str = "llm.guardrail.check";
/// A single LLM inference call — prompt sent, completion received.
///
/// Span: `span.llm.inference`
/// Kind: `client`
/// Stability: `development`
pub const LLM_INFERENCE_SPAN: &str = "llm.inference";
/// LoRA fine-tuning run — applies Low-Rank Adaptation to update a pre-trained model efficiently.
///
/// Span: `span.llm.lora.train`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_LORA_TRAIN_SPAN: &str = "llm.lora.train";
/// Multi-modal LLM processing — handling inputs that combine text with images, audio, video, or documents.
///
/// Span: `span.llm.multimodal.process`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_MULTIMODAL_PROCESS_SPAN: &str = "llm.multimodal.process";
/// Rendering a prompt template — substituting variables to produce the final LLM request payload.
///
/// Span: `span.llm.prompt.render`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_PROMPT_RENDER_SPAN: &str = "llm.prompt.render";
/// Retrieval-augmented generation retrieval step — fetching relevant documents from a vector store.
///
/// Span: `span.llm.rag.retrieve`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_RAG_RETRIEVE_SPAN: &str = "llm.rag.retrieve";
/// LLM response validation — checking a model output against a JSON schema or contract for type safety and completeness.
///
/// Span: `span.llm.response.validate`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_RESPONSE_VALIDATE_SPAN: &str = "llm.response.validate";
/// Configuration of LLM sampling parameters for a generation request.
///
/// Span: `span.llm.sampling.configure`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_SAMPLING_CONFIGURE_SPAN: &str = "llm.sampling.configure";
/// Completion of a streaming LLM response — tracks TTFT, throughput, and chunk delivery.
///
/// Span: `span.llm.streaming.complete`
/// Kind: `client`
/// Stability: `development`
pub const LLM_STREAMING_COMPLETE_SPAN: &str = "llm.streaming.complete";
/// Start of a streaming LLM response — first token received.
///
/// Span: `span.llm.streaming_start`
/// Kind: `client`
/// Stability: `development`
pub const LLM_STREAMING_START_SPAN: &str = "llm.streaming_start";
/// Structured output generation — LLM produces output conforming to a defined schema.
///
/// Span: `span.llm.structured_output.generate`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_STRUCTURED_OUTPUT_GENERATE_SPAN: &str = "llm.structured_output.generate";
/// Token budget enforcement for an LLM session — tracks prompt/completion token usage.
///
/// Span: `span.llm.token.budget`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_TOKEN_BUDGET_SPAN: &str = "llm.token.budget";
/// LLM tool orchestration — coordinates multiple tool calls according to a defined strategy.
///
/// Span: `span.llm.tool.orchestrate`
/// Kind: `internal`
/// Stability: `development`
pub const LLM_TOOL_ORCHESTRATE_SPAN: &str = "llm.tool.orchestrate";

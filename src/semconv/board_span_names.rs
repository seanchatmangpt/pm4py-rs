/// Board chair briefing rendered from L3 intelligence data
///
/// Span: `span.board.briefing_render`
/// Kind: `internal`
/// Stability: `development`
pub const BOARD_BRIEFING_RENDER_SPAN: &str = "board.briefing_render";
/// Conway's Law violation check for a department process
///
/// Span: `span.board.conway_check`
/// Kind: `internal`
/// Stability: `development`
pub const BOARD_CONWAY_CHECK_SPAN: &str = "board.conway_check";
/// Periodic Conway + Little's Law monitoring check summary
///
/// Span: `span.board.conway_check_summary`
/// Kind: `internal`
/// Stability: `development`
pub const BOARD_CONWAY_CHECK_SUMMARY_SPAN: &str = "board.conway_check_summary";
/// Board KPIs computed from process mining event log
///
/// Span: `span.board.kpi_compute`
/// Kind: `internal`
/// Stability: `development`
pub const BOARD_KPI_COMPUTE_SPAN: &str = "board.kpi_compute";
/// Periodic L0 sync — exports BusinessOS cases and handoffs to Oxigraph as RDF facts via bos CLI.
///
/// Span: `span.board.l0_sync`
/// Kind: `internal`
/// Stability: `development`
pub const BOARD_L0_SYNC_SPAN: &str = "board.l0_sync";
/// Board escalation emitted for a structural (Conway) violation
///
/// Span: `span.board.structural_escalation`
/// Kind: `internal`
/// Stability: `development`
pub const BOARD_STRUCTURAL_ESCALATION_SPAN: &str = "board.structural_escalation";

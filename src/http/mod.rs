//! HTTP API module for BusinessOS integration
//!
//! Provides REST API endpoints for process mining operations via axum web framework.
//! Includes both server-side API endpoints and client-side gateway for BusinessOS communication.

pub mod boardchair_api;
pub mod businessos_api;
pub mod businessos_gateway;
pub mod canopy_gateway;
#[cfg(feature = "mcp-server")]
pub mod mcp_state;
pub mod ocel_ingest;
pub mod osa_gateway;
pub mod otel_helpers;

pub use businessos_api::{
    router, ApiError, ConformanceRequest, ConformanceResponse, DiscoveryRequest, DiscoveryResponse,
    HealthResponse, ProgressEvent, StatisticsRequest, StatisticsResponse,
};
pub use businessos_gateway::{
    ActivityStatisticGateway, BusinessOSGateway, CaseDurationStatisticGateway,
    ConformanceGatewayRequest, ConformanceGatewayResponse, DiscoverGatewayRequest,
    DiscoverGatewayResponse, GatewayConfig, GatewayError, GatewayStats, StatisticsGatewayRequest,
    StatisticsGatewayResponse, StatusGatewayResponse,
};
pub use canopy_gateway::{
    CanopyCommitRequest, CanopyCommitResponse, CanopyGateway, CanopyGatewayConfig,
    CanopyGatewayError, CanopyGatewayStats, CanopyPrepareRequest, CanopyPrepareResponse,
    CanopyRollbackRequest, CanopyRollbackResponse,
};
pub use osa_gateway::{
    OsaCommitRequest, OsaCommitResponse, OsaGateway, OsaGatewayConfig, OsaGatewayError,
    OsaGatewayStats, OsaPrepareRequest, OsaPrepareResponse, OsaRollbackRequest,
    OsaRollbackResponse,
};

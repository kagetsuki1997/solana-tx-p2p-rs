// BEGIN LINTS
#![allow(unexpected_cfgs)]
#![cfg_attr(
    feature = "clippy",
    cfg_attr(feature = "c_unwind", deny(ffi_unwind_calls)),
    cfg_attr(feature = "strict_provenance", deny(fuzzy_provenance_casts, lossy_provenance_casts)),
    cfg_attr(feature = "multiple_supertrait_upcastable", deny(multiple_supertrait_upcastable)),
    cfg_attr(feature = "must_not_suspend", deny(must_not_suspend)),
    cfg_attr(feature = "type_privacy_lints", deny(unnameable_types)),
    deny(
      // move to workspace after stabilize
      report-in-external-macro,
      warnings,
    )
)]
// END LINTS
mod app_state;
mod error;
pub mod grpc;
pub mod metrics;
pub mod model;
mod proto;
mod service;
mod shutdown_signal_handler;
pub mod web;

pub use self::{
    app_state::{AppState, DefaultAppState},
    error::{fmt_backtrace, fmt_backtrace_with_source, fmt_source, Error, Result},
    service::PeerWorker,
    shutdown_signal_handler::{ShutdownSignal, SignalHandleBuilder, SignalHandler},
};

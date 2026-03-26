pub(crate) mod kv_gate;
pub(crate) mod capabilities;
pub(crate) mod effect_intents;
pub(crate) mod non_human_policy;
pub(crate) mod non_human_taxonomy;
pub(crate) mod policy_pipeline;
pub(crate) mod policy_graph;
pub(crate) mod policy_taxonomy;
pub(crate) mod traffic_classification;
pub(crate) mod request_outcome;
pub(crate) mod request_facts;
pub(crate) mod request_flow;
pub(crate) mod request_router;
pub(crate) mod sim_telemetry;
pub(crate) mod sim_public;
pub(crate) mod shadow_mode;
pub(crate) mod upstream_canonicalization;
pub(crate) mod upstream_proxy;
pub(crate) mod upstream_telemetry;

#[cfg(test)]
mod architecture_guards;

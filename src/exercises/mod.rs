//! Exercise registry. Each exercise lives in its own module and is
//! registered in [`all`]. Adding a new exercise = three steps:
//! 1. Create `eNN_slug.rs` implementing `Exercise`
//! 2. Declare `mod eNN_slug;` here
//! 3. Add an instance to `all()`
//! trace:STORY-1..17 | ai:claude

use crate::exercise::Exercise;

// ── Core: the 15-minute novice loop (capture → build → link → done).
//    trace:STORY-46 | ai:claude
mod e01_init;
mod e02_feature;
mod e03_list;
mod e04_show;
mod e05_trace_comment;
mod e06_aida_commit;
mod e07_see_link; // the link-bridge "aha" — re-run `aida show` after the commit
mod e08_done;

// ── Going further (optional). Everything past the core loop.
mod e09_in_progress;
mod e10_show_comments;
// The requirement-type tour. trace:STORY-2..6 | ai:claude
mod e11_vision;
mod e12_principle;
mod e13_decision;
mod e14_bug;
mod e15_search;
mod e16_docs_build;
mod e17_status;
mod e18_push;
// Cluster 1 — distributed storage. trace:STORY-25 | ai:claude
mod e19_distributed_store;
mod e20_store_sync;
mod e21_cache_rebuild;
// Cluster 3 — roles + the producer/consumer queue. trace:STORY-27 | ai:claude
mod e22_role_enter;
mod e23_queue_add;
mod e24_queue_pickup;
mod e25_queue_done;
// Cluster 2 — relationships: the requirement graph. trace:STORY-26 | ai:claude
mod e26_add_parent;
mod e27_rel_add;
// Cluster 4 — sessions + worktrees. trace:STORY-28 | ai:claude
mod e28_session_start;
mod e29_session_work;
mod e30_session_leases;
mod e31_session_end;
// Cluster 5 — code review + commit pairing. trace:STORY-29 | ai:claude
mod e32_commit_pair_trailer;
mod e33_review_prompt;
// Cluster 6 — plans + store maintenance + MCP. trace:STORY-30 | ai:claude
mod e34_plan_verify;
mod e35_store_audit;
mod e36_mcp_serve;
// AIDA 0.15 learner-facing coverage. trace:EPIC-7 | ai:codex
mod e37_minimal_why;
mod e38_graph_focus;
mod e39_dispatch_dryrun;
mod e40_advisor_routing;
mod e41_coordination;

pub fn all() -> Vec<Box<dyn Exercise>> {
    vec![
        Box::new(e01_init::E),
        Box::new(e02_feature::E),
        Box::new(e03_list::E),
        Box::new(e04_show::E),
        Box::new(e05_trace_comment::E),
        Box::new(e06_aida_commit::E),
        Box::new(e07_see_link::E),
        Box::new(e08_done::E),
        Box::new(e09_in_progress::E),
        Box::new(e10_show_comments::E),
        Box::new(e11_vision::E),
        Box::new(e12_principle::E),
        Box::new(e13_decision::E),
        Box::new(e14_bug::E),
        Box::new(e15_search::E),
        Box::new(e16_docs_build::E),
        Box::new(e17_status::E),
        Box::new(e18_push::E),
        Box::new(e19_distributed_store::E),
        Box::new(e20_store_sync::E),
        Box::new(e21_cache_rebuild::E),
        Box::new(e22_role_enter::E),
        Box::new(e23_queue_add::E),
        Box::new(e24_queue_pickup::E),
        Box::new(e25_queue_done::E),
        Box::new(e26_add_parent::E),
        Box::new(e27_rel_add::E),
        Box::new(e28_session_start::E),
        Box::new(e29_session_work::E),
        Box::new(e30_session_leases::E),
        Box::new(e31_session_end::E),
        Box::new(e32_commit_pair_trailer::E),
        Box::new(e33_review_prompt::E),
        Box::new(e34_plan_verify::E),
        Box::new(e35_store_audit::E),
        Box::new(e36_mcp_serve::E),
        Box::new(e37_minimal_why::E),
        Box::new(e38_graph_focus::E),
        Box::new(e39_dispatch_dryrun::E),
        Box::new(e40_advisor_routing::E),
        Box::new(e41_coordination::E),
    ]
}

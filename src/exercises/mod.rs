//! Exercise registry. Each exercise lives in its own module and is
//! registered in [`all_exercises`]. Adding a new exercise = three steps:
//! 1. Create `eNN_slug.rs` implementing `Exercise`
//! 2. Declare `mod eNN_slug;` here
//! 3. Add an instance to `all_exercises()`
//! trace:STORY-1..17 | ai:claude

use crate::exercise::Exercise;

mod e01_init;
mod e02_vision;
mod e03_principle;
mod e04_decision;
mod e05_feature;
mod e06_bug;
mod e07_list;
mod e08_show;
mod e09_in_progress;
mod e10_trace_comment;
mod e11_aida_commit;
mod e12_docs_build;
mod e13_search;
mod e14_complete;
mod e15_show_comments;
mod e16_status;
mod e17_push;
// Cluster 1 — distributed storage. trace:STORY-25 | ai:claude
mod e18_distributed_store;
mod e19_store_sync;
mod e20_cache_rebuild;
// Cluster 3 — roles + the producer/consumer queue. trace:STORY-27 | ai:claude
mod e21_role_enter;
mod e22_queue_add;
mod e23_queue_pickup;
mod e24_queue_done;
// Cluster 2 — relationships: the requirement graph. trace:STORY-26 | ai:claude
mod e25_add_parent;
mod e26_rel_add;
// Cluster 4 — sessions + worktrees. trace:STORY-28 | ai:claude
mod e27_session_start;
mod e28_session_work;
mod e29_session_leases;
mod e30_session_end;
// Cluster 5 — code review + commit pairing. trace:STORY-29 | ai:claude
mod e31_commit_pair_trailer;
mod e32_review_prompt;
// Cluster 6 — plans + store maintenance + MCP. trace:STORY-30 | ai:claude
mod e33_plan_verify;
mod e34_store_audit;
mod e35_mcp_serve;

pub fn all() -> Vec<Box<dyn Exercise>> {
    vec![
        Box::new(e01_init::E),
        Box::new(e02_vision::E),
        Box::new(e03_principle::E),
        Box::new(e04_decision::E),
        Box::new(e05_feature::E),
        Box::new(e06_bug::E),
        Box::new(e07_list::E),
        Box::new(e08_show::E),
        Box::new(e09_in_progress::E),
        Box::new(e10_trace_comment::E),
        Box::new(e11_aida_commit::E),
        Box::new(e12_docs_build::E),
        Box::new(e13_search::E),
        Box::new(e14_complete::E),
        Box::new(e15_show_comments::E),
        Box::new(e16_status::E),
        Box::new(e17_push::E),
        Box::new(e18_distributed_store::E),
        Box::new(e19_store_sync::E),
        Box::new(e20_cache_rebuild::E),
        Box::new(e21_role_enter::E),
        Box::new(e22_queue_add::E),
        Box::new(e23_queue_pickup::E),
        Box::new(e24_queue_done::E),
        Box::new(e25_add_parent::E),
        Box::new(e26_rel_add::E),
        Box::new(e27_session_start::E),
        Box::new(e28_session_work::E),
        Box::new(e29_session_leases::E),
        Box::new(e30_session_end::E),
        Box::new(e31_commit_pair_trailer::E),
        Box::new(e32_review_prompt::E),
        Box::new(e33_plan_verify::E),
        Box::new(e34_store_audit::E),
        Box::new(e35_mcp_serve::E),
    ]
}

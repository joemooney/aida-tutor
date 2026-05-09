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
    ]
}

use crate::components::aggregator::SkillMetadata;

#[derive(Debug, Clone)]
pub enum Action {
    Tick,
    Quit,
    NextTab,
    PrevTab,
    ScrollUp,
    ScrollDown,
    Select,
    Refresh,
    ProgressUpdate { value: u64, total: u64, msg: String },
    DataLoaded(Vec<SkillMetadata>),
    Error(String),
}

use crate::components::aggregator::SkillMetadata;
use crate::components::manifest::Repository;
use crate::tui::action::Action;
use crate::tui::views;
use ratatui::{
    layout::Rect,
    Frame,
};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Skills,
    Help,
}

impl Tab {
    pub fn next(self) -> Self {
        match self {
            Self::Dashboard => Self::Skills,
            Self::Skills => Self::Help,
            Self::Help => Self::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Dashboard => Self::Help,
            Self::Skills => Self::Dashboard,
            Self::Help => Self::Skills,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Skills => "Skills Explorer",
            Self::Help => "Help",
        }
    }
}

pub struct TuiApp {
    pub active_tab: Tab,
    pub should_quit: bool,
    
    // Data model
    pub skills: Vec<SkillMetadata>,
    pub repos: Vec<Repository>,
    pub hub_stats: HashMap<String, usize>,
    pub cache_hits: usize,
    pub cache_misses: usize,
    
    // View state
    pub table_scroll_index: usize,
    pub table_selected_index: usize,
    
    // Progress state
    pub is_loading: bool,
    pub loading_value: u64,
    pub loading_total: u64,
    pub loading_msg: String,
    pub error_msg: Option<String>,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            active_tab: Tab::Dashboard,
            should_quit: false,
            skills: Vec::new(),
            repos: Vec::new(),
            hub_stats: HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            table_scroll_index: 0,
            table_selected_index: 0,
            is_loading: false,
            loading_value: 0,
            loading_total: 100,
            loading_msg: String::new(),
            error_msg: None,
        }
    }

    // A lightweight load, just what we can read synchronously for now
    pub fn load_data(&mut self, repo_root: &Path, repos: Vec<Repository>) -> anyhow::Result<()> {
        self.repos = repos;
        
        // 1. Load Cache Stats
        if let Ok(cache_map) = crate::components::llm::load_cache() {
            let metrics = crate::components::llm::cache_metrics();
            self.cache_hits = metrics.hits;
            self.cache_misses = metrics.misses;
            // Also store total entries as a stat
            self.hub_stats.insert("Cache Size".to_string(), cache_map.len());
        }

        // 2. Load Aggregated Hub Stats from CSV
        let csv_path = repo_root.join("skills-aggregated").join("hub-manifests.csv");
        if csv_path.exists() {
            if let Ok(mut rdr) = csv::Reader::from_path(&csv_path) {
                let mut counts = HashMap::new();
                for result in rdr.records() {
                    if let Ok(record) = result {
                        if let Some(hub) = record.get(0) {
                            *counts.entry(hub.to_string()).or_insert(0) += 1;
                        }
                    }
                }
                self.hub_stats.extend(counts);
            }
        }

        Ok(())
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::NextTab => self.active_tab = self.active_tab.next(),
            Action::PrevTab => self.active_tab = self.active_tab.prev(),
            Action::ScrollDown => {
                if self.active_tab == Tab::Skills && !self.skills.is_empty() {
                    if self.table_selected_index < self.skills.len() - 1 {
                        self.table_selected_index += 1;
                    }
                }
            }
            Action::ScrollUp => {
                if self.active_tab == Tab::Skills {
                    self.table_selected_index = self.table_selected_index.saturating_sub(1);
                }
            }
            Action::ProgressUpdate { value, total, msg } => {
                self.is_loading = true;
                self.loading_value = value;
                self.loading_total = total;
                self.loading_msg = msg;
                if value >= total && total > 0 {
                    self.is_loading = false;
                }
            }
            Action::DataLoaded(new_skills) => {
                self.skills = new_skills;
                self.is_loading = false;
                self.table_selected_index = 0;
                self.table_scroll_index = 0;
                
                // Refresh hub stats from memory instead of just CSV
                let mut counts = HashMap::new();
                for skill in &self.skills {
                    *counts.entry(skill.hub.clone()).or_insert(0) += 1;
                }
                // Merge with existing (preserve "Cache Size" etc)
                for (hub, count) in counts {
                    self.hub_stats.insert(hub, count);
                }
            }
            Action::Error(msg) => {
                self.error_msg = Some(msg);
                self.is_loading = false;
            }
            _ => {} // Tick, Select, Refresh handled by specific views if needed
        }
    }

    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        
        // Pass self immutably to views based on active tab
        match self.active_tab {
            Tab::Dashboard => views::dashboard::render(self, f, area),
            Tab::Skills => views::skills::render(self, f, area),
            Tab::Help => views::help::render(self, f, area),
        }
    }
}

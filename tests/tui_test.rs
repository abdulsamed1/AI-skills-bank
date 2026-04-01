use skill_manage::tui::app::{TuiApp, Tab};
use skill_manage::tui::action::Action;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_tab_navigation() {
    let mut app = TuiApp::new();
    assert_eq!(app.active_tab, Tab::Dashboard);

    app.update(Action::NextTab);
    assert_eq!(app.active_tab, Tab::Skills);

    app.update(Action::NextTab);
    assert_eq!(app.active_tab, Tab::Help);

    app.update(Action::NextTab);
    assert_eq!(app.active_tab, Tab::Dashboard);

    app.update(Action::PrevTab);
    assert_eq!(app.active_tab, Tab::Help);
}

#[test]
fn test_quit_action() {
    let mut app = TuiApp::new();
    assert!(!app.should_quit);

    app.update(Action::Quit);
    assert!(app.should_quit);
}

#[test]
fn test_progress_update() {
    let mut app = TuiApp::new();
    assert!(!app.is_loading);

    app.update(Action::ProgressUpdate {
        value: 50,
        total: 100,
        msg: "Loading...".to_string(),
    });

    assert!(app.is_loading);
    assert_eq!(app.loading_value, 50);
    assert_eq!(app.loading_total, 100);
    assert_eq!(app.loading_msg, "Loading...");

    app.update(Action::ProgressUpdate {
        value: 100,
        total: 100,
        msg: "Done".to_string(),
    });
    assert!(!app.is_loading);
}

#[test]
fn test_rendering_dashboard() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = TuiApp::new();

    terminal.draw(|f| app.render(f)).unwrap();

    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer);
    
    // Assert that the dashboard title is rendered
    assert!(content.contains("Dashboard") || content.contains("D a s h b o a r d"));
}

#[test]
fn test_rendering_help() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = TuiApp::new();

    app.active_tab = Tab::Help;
    terminal.draw(|f| app.render(f)).unwrap();

    let buffer = terminal.backend().buffer();
    let content = format!("{:?}", buffer);
    
    // Assert that the help title and content is present
    assert!(content.contains("Help") || content.contains("H e l p"));
    assert!(content.contains("Quit") || content.contains("Q u i t"));
}

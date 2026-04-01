use skill_manage::components::fetcher::Fetcher;
use skill_manage::components::manifest::{RepoManifest, Repository};

#[test]
fn test_normalize_repo_url_collapses_common_variants() {
    let a = Fetcher::normalize_repo_url("https://github.com/Org/Repo.git");
    let b = Fetcher::normalize_repo_url("https://github.com/org/repo/");
    assert_eq!(a, b);
}

#[test]
fn test_dedupe_manifest_repositories_skips_duplicate_names_and_urls() {
    let manifest = RepoManifest {
        repositories: vec![
            Repository {
                name: "repo-a".to_string(),
                url: "https://github.com/org/repo-a.git".to_string(),
                branch: None,
            },
            Repository {
                // Duplicate URL (different formatting) should be skipped.
                name: "repo-a-alt".to_string(),
                url: "https://github.com/org/repo-a/".to_string(),
                branch: None,
            },
            Repository {
                // Duplicate name should be skipped.
                name: "repo-a".to_string(),
                url: "https://github.com/org/repo-b.git".to_string(),
                branch: None,
            },
            Repository {
                name: "repo-c".to_string(),
                url: "https://github.com/org/repo-c.git".to_string(),
                branch: None,
            },
        ],
    };

    let deduped = Fetcher::dedupe_manifest_repositories(&manifest);
    let names = deduped.iter().map(|r| r.name.as_str()).collect::<Vec<_>>();

    assert_eq!(deduped.len(), 2);
    assert_eq!(names, vec!["repo-a", "repo-c"]);
}

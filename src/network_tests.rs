use super::UrlMatch;

#[test]
fn url_matches_follow_exact_contains_and_glob_rules() {
    let url = "https://shop.test/api/cart?id=1";
    assert!(UrlMatch::Exact(url.into()).matches(url));
    assert!(!UrlMatch::Exact("https://shop.test/api/cart".into()).matches(url));
    assert!(UrlMatch::Contains("/api/".into()).matches(url));
    assert!(UrlMatch::Glob("**/api/cart?id=?".into()).matches(url));
    assert!(UrlMatch::Glob("https://shop.test/*/cart*".into()).matches(url));
    assert!(!UrlMatch::Glob("https://shop.test/*".into()).matches(url));
    assert!(UrlMatch::Glob("https://shop.test/**".into()).matches(url));
}

#[test]
fn a_backtracking_glob_finishes_quickly() {
    let text = format!("https://x.test/{}", "a".repeat(4000));
    let started = std::time::Instant::now();
    assert!(!UrlMatch::Glob("**a*a*a*a*a*a*a*a*b".into()).matches(&text));
    assert!(started.elapsed() < std::time::Duration::from_millis(200));
}

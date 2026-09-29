use weft::render::{RenderError, render_html};

const SOURCE: &str = r#"site Acme & Sons
page /:
  hero:
    title "Build " + "<native> sites"
    text "No framework & " + "no runtime."
    actions:
      "Start " + "free" -> /signup primary
  section features cards 2:
    card "Native HTML" "Useful without JavaScript."
    card "Raw CSS" "No utility-class output."
"#;

#[test]
fn renders_semantic_escaped_html() {
    let html = render_html(SOURCE).expect("valid source should render");

    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<main>"));
    assert!(html.contains("<h1 class=\"weft-title\">Build &lt;native&gt; sites</h1>"));
    assert!(html.contains("No framework &amp; no runtime."));
    assert!(html.contains("href=\"/signup\""));
    assert!(html.contains("<article class=\"weft-card\">"));
    assert!(!html.contains("react"));
}

#[test]
fn rejects_unsafe_link_destinations() {
    let source = r#"site Acme
page /:
  hero:
    actions:
      "Nope" -> javascript:alert(1) primary
"#;

    let error = render_html(source).expect_err("javascript URLs must be rejected");
    assert_eq!(
        error,
        RenderError::UnsafeDestination {
            line: 5,
            destination: "javascript:alert(1)".to_owned()
        }
    );
}

#[test]
fn requires_site_and_one_root_page() {
    assert_eq!(
        render_html("page /:\n").expect_err("site is required"),
        RenderError::MissingSite
    );

    let error = render_html("site Acme\npage /about:\n")
        .expect_err("only root pages are currently renderable");
    assert_eq!(error, RenderError::UnsupportedPages);
}

#[test]
fn rejects_state_text_outside_an_island() {
    let source = r#"site Acme
page /:
  hero:
    title seats + " seats"
"#;

    let error = render_html(source).expect_err("page text has no state scope");
    assert_eq!(error, RenderError::DynamicTextOutsideIsland { line: 4 });
}

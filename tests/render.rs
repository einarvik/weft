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

#[test]
fn renders_accessible_card_images() {
    let source = r#"site Acme
page /:
  section work cards 2:
    card "First" "First image loads eagerly." image "/images/first.webp" alt "First project"
    card "Second" "Second image loads later." image "https://images.example/second.webp" alt "Second project"
"#;

    let html = render_html(source).expect("card images should render");
    assert!(html.contains("src=\"/images/first.webp\" alt=\"First project\""));
    assert!(html.contains(
        "src=\"https://images.example/second.webp\" alt=\"Second project\" loading=\"lazy\""
    ));
}

#[test]
fn renders_semantic_standalone_images_with_optional_captions() {
    let source = r#"site Acme
page /:
  section story:
    image "/images/workspace.webp" alt "A & calm <workspace>" caption "Acme & partners."
    image "https://images.example/team.webp" alt "A focused team"
"#;

    let html = render_html(source).expect("standalone images should render");
    assert!(html.contains("<figure class=\"weft-image\">"));
    assert!(html.contains("src=\"/images/workspace.webp\" alt=\"A &amp; calm &lt;workspace&gt;\""));
    assert!(html.contains("<figcaption>Acme &amp; partners.</figcaption>"));
    assert_eq!(html.matches("<figcaption>").count(), 1);
    assert!(!html.contains("loading=\"lazy\""));
    assert!(!html.contains("<script"));
}

#[test]
fn rejects_unsafe_card_image_urls() {
    let source = r#"site Acme
page /:
  section work cards 1:
    card "Unsafe" "Never render this." image "javascript:alert(1)" alt "Unsafe"
"#;

    let error = render_html(source).expect_err("unsafe image URL must fail");
    assert_eq!(
        error,
        RenderError::UnsafeImageSource {
            line: 4,
            url: "javascript:alert(1)".to_owned()
        }
    );
}

#[test]
fn rejects_unsafe_standalone_image_urls() {
    let source = r#"site Acme
page /:
  section story:
    image "javascript:alert(1)" alt "Unsafe"
"#;

    let error = render_html(source).expect_err("unsafe image URL must fail");
    assert_eq!(
        error,
        RenderError::UnsafeImageSource {
            line: 4,
            url: "javascript:alert(1)".to_owned()
        }
    );
}

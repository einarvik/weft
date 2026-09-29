use weft::{island::render_javascript, parse, render::render_html, style::render_css};

const COUNTER: &str = r#"site Acme
page /:
  island counter:
    label seats + " Seats"
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
"#;

#[test]
fn emits_a_module_only_for_explicit_islands() {
    let document = parse(COUNTER).expect("counter source should parse");
    let javascript = render_javascript(&document)
        .expect("counter should compile")
        .expect("a counter needs browser code");
    let html = render_html(COUNTER).expect("counter HTML should render");
    let css = render_css(COUNTER).expect("counter CSS should render");

    assert!(javascript.contains("customElements.define(\"weft-counter\""));
    assert!(html.contains("<weft-counter"));
    assert!(html.contains("src=\"islands.js\""));
    assert!(html.contains(
        "data-label-expression=\"[[&quot;s&quot;,0],[&quot;l&quot;,&quot; Seats&quot;]]\""
    ));
    assert!(html.contains("data-text-expression=\"[[&quot;l&quot;,&quot;$&quot;],[&quot;m&quot;,12],[&quot;l&quot;,&quot;/month&quot;]]\""));
    assert!(css.contains(".weft-island"));
}

#[test]
fn static_documents_emit_no_javascript() {
    let document = parse("site Acme\npage /:\n  hero:\n    title \"Static\"\n")
        .expect("static source should parse");

    assert_eq!(
        render_javascript(&document).expect("static source should validate"),
        None
    );
}

#[test]
fn rejects_expression_outside_the_supported_counter_subset() {
    let source = r#"site Acme
page /:
  island counter:
    state seats=5
    range seats 1..100
    text people + " seats"
"#;

    let error = render_html(source).expect_err("arbitrary expressions must not compile");
    assert!(error.to_string().contains("only state `seats`"));
}

#[test]
fn mounts_named_islands_inside_semantic_sections() {
    let source = r#"site Acme
page /:
  island seat-price counter:
    label seats + " Seats"
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
  section pricing @="wrap:reading gap:lg":
    eyebrow "Pricing"
    title "Pay for the seats you need."
    text "Change the team size to see your monthly cost."
    use seat-price
"#;

    let html = render_html(source).expect("named island should render when mounted");
    let css = render_css(source).expect("named island CSS should render");

    assert!(html.contains("<h2 class=\"weft-section-title\">Pay for the seats you need.</h2>"));
    assert!(html.contains("<p class=\"weft-section-text\">Change the team size"));
    assert_eq!(html.matches("<weft-counter").count(), 1);
    assert!(html.contains("src=\"islands.js\""));
    assert!(css.contains(".weft-island"));
}

#[test]
fn named_island_declarations_need_a_mount() {
    let source = r#"site Acme
page /:
  island seat-price counter:
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
"#;

    let html = render_html(source).expect("unused declaration is valid");
    assert!(!html.contains("<weft-counter"));
    assert!(!html.contains("islands.js"));
}

#[test]
fn rejects_unknown_named_islands() {
    let source = r#"site Acme
page /:
  section pricing:
    use missing-counter
"#;

    let error = render_html(source).expect_err("unknown island references must fail");
    assert!(
        error
            .to_string()
            .contains("unknown island reference `missing-counter`")
    );
}

#[test]
fn mounts_inline_islands_inside_semantic_sections() {
    let source = r#"site Acme
page /:
  section pricing:
    title "Pricing"
    island counter:
      label seats + " Seats"
      state seats=5
      range seats 1..100
      text "$" + seats * 12 + "/month"
"#;

    let html = render_html(source).expect("inline island should render");
    assert!(html.contains("<section class=\"weft-section weft-section--pricing\""));
    assert!(html.contains("<weft-counter"));
}

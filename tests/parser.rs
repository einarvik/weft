use weft::{Block, SectionChild, SectionKind, parse};

const SOURCE: &str = r#"site Acme
theme: brand violet; ink slate; radius lg

page /:
  hero:
    eyebrow "For independent teams"
    title "Ship projects without the ceremony."
    text "Plan, discuss, and deliver from one calm workspace."
    actions:
      "Start free" -> /signup primary
      "See demo" -> #demo quiet

  section features cards 3 @="wrap:wide gap:lg surface:soft":
    card "One source of truth" "Keep context close to the work."
    card "Built for momentum" "Move from idea to delivery."

  island counter:
    label "Seats"
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
"#;

#[test]
fn parses_the_planned_document_shape() {
    let document = parse(SOURCE).expect("the source should parse");

    assert_eq!(document.site.expect("site").name, "Acme");
    assert_eq!(document.theme["brand"], "violet");
    assert_eq!(document.pages.len(), 1);

    let page = &document.pages[0];
    assert_eq!(page.path, "/");
    assert_eq!(page.blocks.len(), 3);

    let Block::Hero(hero) = &page.blocks[0] else {
        panic!("first block should be a hero");
    };
    assert_eq!(hero.actions.len(), 2);
    assert_eq!(hero.actions[0].destination, "/signup");

    let Block::Section(section) = &page.blocks[1] else {
        panic!("second block should be a section");
    };
    let SectionKind::Cards { columns, items } = &section.kind else {
        panic!("section should be a card grid");
    };
    assert_eq!(*columns, 3);
    assert_eq!(
        section.style.as_deref(),
        Some("wrap:wide gap:lg surface:soft")
    );
    assert_eq!(items.len(), 2);

    let Block::Island(island) = &page.blocks[2] else {
        panic!("third block should be an island");
    };
    assert_eq!(island.state.as_ref().expect("state").initial, 5);
}

#[test]
fn parses_semantic_sections_and_named_island_references() {
    let source = r#"site Acme
page /:
  island seat-price counter:
    state seats=5
    range seats 1..100
    text "$" + seats * 12 + "/month"
  section pricing @="wrap:reading gap:lg":
    eyebrow "Pricing"
    title "Pay for " + "your team."
    text "Choose the seats you need."
    use seat-price
"#;

    let document = parse(source).expect("semantic section source should parse");
    let page = &document.pages[0];
    assert_eq!(page.named_islands.len(), 1);
    assert!(page.named_islands.contains_key("seat-price"));
    let Block::Section(section) = &page.blocks[0] else {
        panic!("visible block should be a section");
    };
    let SectionKind::Content { children } = &section.kind else {
        panic!("section should contain semantic children");
    };
    assert!(matches!(children[0], SectionChild::Eyebrow(_)));
    assert!(matches!(children[3], SectionChild::Use { ref name, .. } if name == "seat-price"));
}

#[test]
fn reports_the_line_for_invalid_indentation() {
    let error = parse("site Acme\n page /:\n").expect_err("odd indentation should fail");

    assert_eq!(error.line, 2);
    assert!(error.message.contains("two-space"));
}

#[test]
fn rejects_unknown_and_nested_statements() {
    let unknown = parse("site Acme\nwat\n").expect_err("unknown statement should fail");
    assert_eq!(unknown.line, 2);

    let nested = parse("page /:\n    hero:\n").expect_err("jumped indentation should fail");
    assert_eq!(nested.line, 2);
    assert!(nested.message.contains("indentation"));
}

#[test]
fn rejects_invalid_card_counts() {
    let error =
        parse("page /:\n  section features cards 0:\n").expect_err("zero card count should fail");

    assert_eq!(error.line, 2);
    assert!(error.message.contains("section syntax"));
}

use faf_browser::dom::HtmlDocument;
use faf_browser::render::layout::compute_layout;
use faf_browser::render::tree::build_layout_tree;
use faf_browser::css::parser::parse_css;
use faf_browser::css::style::compute_styles;

#[test]
fn debug_books_toscrape_layout() {
    let html = r#"
<!DOCTYPE html>
<html><body>
<div class="container-fluid page">
    <div class="page_inner">
        <div class="row">
            <aside class="sidebar col-sm-4 col-md-3">
                <div class="side_categories">
                    <ul class="nav nav-list">
                        <li><a href="catalogue/category/books_1/index.html">Books</a>
                        <ul><li><a href="catalogue/category/books/travel_2/index.html">Travel</a></li></ul>
                        </li>
                    </ul>
                </div>
            </aside>
            <div class="col-sm-8 col-md-9">
                <h1>All products</h1>
                <section>
                    <div class="alert alert-warning" role="alert"><strong>Warning!</strong> This is a demo.</div>
                    <div>
                        <ol class="row">
                            <li class="col-xs-6 col-sm-4 col-md-3 col-lg-3">
                                <article class="product_pod">
                                    <div class="image_container">
                                        <a href="catalogue/a-light-in-the-attic_1000/index.html">
                                            <img src="media/cache/.../a-light-in-the-attic.jpg" alt="A Light in the Attic" class="thumbnail">
                                        </a>
                                    </div>
                                    <p class="star-rating Three"><i class="icon-star"></i></p>
                                    <h3><a href="..." title="A Light in the Attic">A Light in the ...</a></h3>
                                    <div class="product_price">
                                        <p class="price_color">£51.77</p>
                                        <p class="instock availability"><i class="icon-ok"></i> In stock</p>
                                        <form><button class="btn btn-primary btn-block" type="submit">Add to basket</button></form>
                                    </div>
                                </article>
                            </li>
                        </ol>
                    </div>
                </section>
            </div>
        </div>
    </div>
</div>
</body></html>
"#;

    let doc = HtmlDocument::parse(html);
    let css = r#"
.container-fluid { padding-right: 15px; padding-left: 15px; margin-right: auto; margin-left: auto; }
.row { margin-left: -15px; margin-right: -15px; }
.col-sm-4, .col-sm-8, .col-md-3, .col-md-9 { position: relative; min-height: 1px; padding-left: 15px; padding-right: 15px; }
.col-sm-8 { width: 66.66666667%; }
.col-sm-4 { width: 33.33333333%; }
.sidebar { padding: 20px; background: #f5f5f5; }
h1 { font-size: 24px; }
.product_pod { padding: 10px; border: 1px solid #ddd; }
.image_container { height: 200px; }
.thumbnail { width: 100%; height: auto; }
.price_color { color: #b12704; font-size: 18px; }
.btn { padding: 6px 12px; background: #5cb85c; color: white; }
"#;
    let sheet = parse_css(css).unwrap();
    let computed = compute_styles(&doc, &sheet);
    let mut tree = build_layout_tree(&doc, &computed, None);
    compute_layout(&mut tree, 1280.0);

    fn print_tree(node: &faf_browser::render::tree::VisualNode, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}{} ({:.1}x{:.1} @ {:.1},{:.1}) text='{}' display='{}' children={}",
            indent,
            node.tag,
            node.rect.width(),
            node.rect.height(),
            node.rect.x(),
            node.rect.y(),
            if node.text.len() > 30 { &node.text[..30] } else { &node.text },
            node.style.display,
            node.children.len()
        );
        for child in &node.children {
            print_tree(child, depth + 1);
        }
    }

    println!("=== Layout Tree ===");
    print_tree(&tree, 0);
}

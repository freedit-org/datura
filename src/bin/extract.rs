use std::fs;

use once_cell::sync::Lazy;
use scraper::{Html, Selector};

const TITLE_STR: &str = r#"title"#;
const COVER_STR: &str = r#"a[class="entity-detail__img-origin"]"#;
const H5_STR: &str = r#"h5"#;
const A_STR: &str = r#"a"#;
const DETAILS_STR: &str = r#"div[class="entity-detail__fields"]"#;
const DIV_STR: &str = r#"div"#;
const SPAN_STR: &str = r#"span"#;
const TAG_STR: &str = r#"span[class="tag-collection__tag"]"#;
const DESC_STR: &str = r#"p[class="entity-desc__content"]"#;
static TITLE: Lazy<Selector> = Lazy::new(|| Selector::parse(TITLE_STR).unwrap());
static COVER: Lazy<Selector> = Lazy::new(|| Selector::parse(COVER_STR).unwrap());
static H5: Lazy<Selector> = Lazy::new(|| Selector::parse(H5_STR).unwrap());
static A: Lazy<Selector> = Lazy::new(|| Selector::parse(A_STR).unwrap());
static DETAILS: Lazy<Selector> = Lazy::new(|| Selector::parse(DETAILS_STR).unwrap());
static DIV: Lazy<Selector> = Lazy::new(|| Selector::parse(DIV_STR).unwrap());
static SPAN: Lazy<Selector> = Lazy::new(|| Selector::parse(SPAN_STR).unwrap());
static TAG: Lazy<Selector> = Lazy::new(|| Selector::parse(TAG_STR).unwrap());
static DESC: Lazy<Selector> = Lazy::new(|| Selector::parse(DESC_STR).unwrap());

fn main() {
    let html = fs::read_to_string("1.html").unwrap();
    let book = Book::from(html.as_str());
    dbg!(&book);
}

#[derive(Debug)]
struct Book {
    title: String,
    cover: Option<String>,
    source: Option<String>,
    isbn: Option<String>,
    authors: Vec<String>,
    publisher: Option<String>,
    subtitle: Option<String>,
    translators: Vec<String>,
    original_title: Option<String>,
    language: Option<String>,
    pub_time: Option<String>,
    bookformat: Option<String>,
    price: Option<String>,
    pages: Option<String>,
    other_info: Option<String>,
    tags: Vec<String>,
    description: String,
    content: String,
}

impl From<&str> for Book {
    fn from(html: &str) -> Self {
        let fragment = Html::parse_fragment(&html);

        let title = fragment
            .select(&TITLE)
            .next()
            .unwrap()
            .inner_html()
            .rsplit_once("| ")
            .unwrap()
            .1
            .to_owned();

        let cover = fragment
            .select(&COVER)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .map(|s| s.to_owned());

        let source = fragment
            .select(&H5)
            .next()
            .unwrap()
            .select(&A)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .map(|s| s.to_owned());

        // details
        let mut details = fragment.select(&DETAILS);

        // first parts
        let mut div_ele = details.next().unwrap().select(&DIV).skip(1);

        let isbn = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("ISBN：").to_owned());

        let mut authors = vec![];
        let span = Selector::parse(r#"span"#).unwrap();
        for s in div_ele.next().unwrap().select(&span) {
            authors.push(s.inner_html())
        }

        let publisher = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("出版社：").to_owned());

        let subtitle = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("副标题：").to_owned());

        let mut translators = vec![];
        div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .for_each(|ele| translators.push(ele.inner_html()));

        let original_title = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("原作名：").to_owned());

        let language = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("语言：").to_owned());

        let pub_time = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("出版时间：").to_owned());

        // second parts
        let mut div_ele = details.next().unwrap().select(&DIV);

        let bookformat = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("装帧：").to_owned());

        let price = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("定价：").to_owned());

        let pages = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("页数：").to_owned());

        let other_info = div_ele.next().map(|ele| ele.inner_html().trim().to_owned());

        let mut tags = vec![];
        for i in fragment.select(&TAG) {
            let tag = i.select(&A).next().unwrap().inner_html();
            tags.push(tag);
        }

        let mut div_ele = fragment.select(&DESC);
        let description = div_ele.next().unwrap().inner_html().trim().to_owned();

        let content = div_ele.next().unwrap().inner_html().trim().to_owned();

        Book {
            title,
            cover,
            source,
            isbn,
            authors,
            publisher,
            subtitle,
            translators,
            original_title,
            language,
            pub_time,
            bookformat,
            price,
            pages,
            other_info,
            tags,
            description,
            content,
        }
    }
}

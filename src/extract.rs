use bincode::{Decode, Encode};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};

const TITLE_STR: &str = r#"title"#;
const H5_STR: &str = r#"h5"#;
const A_STR: &str = r#"a"#;
const DIV_STR: &str = r#"div"#;
const SPAN_STR: &str = r#"span"#;

const COVER_STR: &str = r#"a[class="entity-detail__img-origin"]"#;
const DETAILS_STR: &str = r#"div[class="entity-detail__fields"]"#;
const TAG_STR: &str = r#"span[class="tag-collection__tag"]"#;
const DESC_STR: &str = r#"p[class="entity-desc__content"]"#;

const DIRECTOR_STR: &str = r#"span[class="director"]"#;
const PLAYWRIGHT_STR: &str = r#"span[class="playwright"]"#;
const ACTOR_STR: &str = r#"span[class="actor"]"#;

const ARTIST_STR: &str = r#"span[class="artist"]"#;
const COMPANY_STR: &str = r#"span[class="company"]"#;
const TRACK_STR: &str = r#"span[class="track-carousel__track-title"]"#;

static TITLE: Lazy<Selector> = Lazy::new(|| Selector::parse(TITLE_STR).unwrap());
static H5: Lazy<Selector> = Lazy::new(|| Selector::parse(H5_STR).unwrap());
static A: Lazy<Selector> = Lazy::new(|| Selector::parse(A_STR).unwrap());
static DIV: Lazy<Selector> = Lazy::new(|| Selector::parse(DIV_STR).unwrap());
static SPAN: Lazy<Selector> = Lazy::new(|| Selector::parse(SPAN_STR).unwrap());

static COVER: Lazy<Selector> = Lazy::new(|| Selector::parse(COVER_STR).unwrap());
static DETAILS: Lazy<Selector> = Lazy::new(|| Selector::parse(DETAILS_STR).unwrap());
static TAG: Lazy<Selector> = Lazy::new(|| Selector::parse(TAG_STR).unwrap());
static DESC: Lazy<Selector> = Lazy::new(|| Selector::parse(DESC_STR).unwrap());

static DIRECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(DIRECTOR_STR).unwrap());
static PLAYWRIGHT: Lazy<Selector> = Lazy::new(|| Selector::parse(PLAYWRIGHT_STR).unwrap());
static ACTOR: Lazy<Selector> = Lazy::new(|| Selector::parse(ACTOR_STR).unwrap());

static ARTIST: Lazy<Selector> = Lazy::new(|| Selector::parse(ARTIST_STR).unwrap());
static COMPANY: Lazy<Selector> = Lazy::new(|| Selector::parse(COMPANY_STR).unwrap());
static TRACK: Lazy<Selector> = Lazy::new(|| Selector::parse(TRACK_STR).unwrap());

#[derive(Debug, Encode, Decode)]
pub struct Book {
    pub title: String,
    pub cover: Option<String>,
    pub source: Option<String>,
    pub isbn: Option<String>,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub subtitle: Option<String>,
    pub translators: Vec<String>,
    pub original_title: Option<String>,
    pub language: Option<String>,
    pub pub_time: Option<String>,
    pub bookformat: Option<String>,
    pub price: Option<String>,
    pub pages: Option<String>,
    pub other_info: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub content: Option<String>,
}

impl From<&str> for Book {
    fn from(html: &str) -> Self {
        let fragment = Html::parse_fragment(html);

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
            .map(|s| s.to_owned())
            .and_then(empty2none);

        let source = fragment
            .select(&H5)
            .next()
            .unwrap()
            .select(&A)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .map(|s| s.to_owned())
            .and_then(empty2none);

        // details
        let mut details = fragment.select(&DETAILS);

        // first parts
        let mut div_ele = details.next().unwrap().select(&DIV).skip(1);

        let isbn = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("ISBN：").to_owned())
            .and_then(empty2none);

        let authors = div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        let publisher = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("出版社：").to_owned())
            .and_then(empty2none);

        let subtitle = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("副标题：").to_owned())
            .and_then(empty2none);

        let translators = div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        let original_title = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("原作名：").to_owned())
            .and_then(empty2none);

        let language = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("语言：").to_owned())
            .and_then(empty2none);

        let pub_time = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("出版时间：").to_owned())
            .and_then(empty2none);

        // second parts
        let mut div_ele = details.next().unwrap().select(&DIV);

        let bookformat = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("装帧：").to_owned())
            .and_then(empty2none);

        let price = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("定价：").to_owned())
            .and_then(empty2none);

        let pages = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("页数：").to_owned())
            .and_then(empty2none);

        let other_info = div_ele
            .next()
            .map(|ele| ele.inner_html().trim().to_owned())
            .and_then(empty2none);

        let tags = fragment
            .select(&TAG)
            .map(|ele| ele.select(&A).next().unwrap().inner_html())
            .collect();

        let mut div_ele = fragment.select(&DESC);
        let description = div_ele
            .next()
            .map(|ele| ele.inner_html().trim().to_owned())
            .and_then(empty2none);

        let content = div_ele
            .next()
            .map(|ele| ele.inner_html().trim().to_owned())
            .and_then(empty2none);

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

#[derive(Debug, Encode, Decode)]
pub struct Movie {
    pub title: String,
    pub cover: Option<String>,
    pub source: Option<String>,
    pub imdb: Option<String>,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
    pub stars: Vec<String>,
    pub genres: Vec<String>,
    pub countries: Vec<String>,
    pub languages: Vec<String>,
    pub runtime: Option<String>,
    pub season: Option<String>,
    pub episode: Option<String>,
    pub episode_runtime: Option<String>,
    pub release_dates: Vec<String>,
    pub alias: Vec<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

impl From<&str> for Movie {
    fn from(html: &str) -> Self {
        let fragment = Html::parse_fragment(html);

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
            .map(|s| s.to_owned())
            .and_then(empty2none);

        let source = fragment
            .select(&H5)
            .next()
            .unwrap()
            .select(&A)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .map(|s| s.to_owned())
            .and_then(empty2none);

        // details
        let mut details = fragment.select(&DETAILS);

        // first parts
        let mut div_ele = details.next().unwrap().select(&DIV).skip(1);

        let imdb = div_ele
            .next()
            .and_then(|ele| ele.select(&A).next().map(|a| a.inner_html()))
            .and_then(empty2none);

        let directors = fragment
            .select(&DIRECTOR)
            .map(|ele| ele.inner_html())
            .collect();

        let writers = fragment
            .select(&PLAYWRIGHT)
            .map(|ele| ele.inner_html())
            .collect();

        let stars = fragment
            .select(&ACTOR)
            .map(|ele| ele.inner_html())
            .collect();

        let genres = div_ele
            .nth(3)
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        let countries = div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        let languages = div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        // second parts
        let mut div_ele = details.next().unwrap().select(&DIV);

        let runtime = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("片长：").to_owned())
            .and_then(empty2none);

        let season = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("季数：").to_owned())
            .and_then(empty2none);

        let episode = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("集数：").to_owned())
            .and_then(empty2none);

        let episode_runtime = div_ele
            .next()
            .map(|ele| ele.inner_html().trim_start_matches("单集长度：").to_owned())
            .and_then(empty2none);

        let release_dates = div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        let alias = div_ele
            .next()
            .unwrap()
            .select(&SPAN)
            .map(|ele| ele.inner_html())
            .collect();

        let tags = fragment
            .select(&TAG)
            .map(|ele| ele.select(&A).next().unwrap().inner_html())
            .collect();

        let mut div_ele = fragment.select(&DESC);
        let description = div_ele
            .next()
            .map(|ele| ele.inner_html().trim().to_owned())
            .and_then(empty2none);

        Movie {
            title,
            cover,
            source,
            imdb,
            directors,
            writers,
            stars,
            genres,
            countries,
            languages,
            runtime,
            season,
            episode,
            episode_runtime,
            release_dates,
            alias,
            tags,
            description,
        }
    }
}

#[derive(Debug, Encode, Decode)]
pub struct Album {
    pub title: String,
    pub cover: Option<String>,
    pub source: Option<String>,
    pub artists: Vec<String>,
    pub companies: Vec<String>,
    pub pub_time: Option<String>,
    pub genre: Option<String>,
    pub medium: Option<String>,
    pub code: Option<String>,
    pub format: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tracks: Vec<String>,
}

impl From<&str> for Album {
    fn from(html: &str) -> Self {
        let fragment = Html::parse_fragment(html);

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
            .map(|s| s.to_owned())
            .and_then(empty2none);

        let source = fragment
            .select(&H5)
            .next()
            .unwrap()
            .select(&A)
            .next()
            .unwrap()
            .value()
            .attr("href")
            .map(|s| s.to_owned())
            .and_then(empty2none);

        // details
        let mut details = fragment.select(&DETAILS);

        // first parts
        let mut div_ele = details.next().unwrap().select(&DIV).skip(1);

        let artists = fragment
            .select(&ARTIST)
            .map(|ele| ele.inner_html())
            .collect();

        let companies = fragment
            .select(&COMPANY)
            .map(|ele| ele.inner_html())
            .collect();

        let pub_time = div_ele
            .nth(2)
            .map(|ele| {
                ele.inner_html()
                    .trim()
                    .trim_start_matches("发行日期：")
                    .to_owned()
            })
            .and_then(empty2none);

        let genre = div_ele
            .nth(1)
            .map(|ele| {
                ele.inner_html()
                    .trim()
                    .trim_start_matches("流派：")
                    .to_owned()
            })
            .and_then(empty2none);

        // second parts
        let mut div_ele = details.next().unwrap().select(&DIV);

        let medium = div_ele
            .next()
            .map(|ele| {
                ele.inner_html()
                    .trim()
                    .trim_start_matches("介质：")
                    .to_owned()
            })
            .and_then(empty2none);

        let code = div_ele
            .next()
            .map(|ele| {
                ele.inner_html()
                    .trim()
                    .trim_start_matches("条形码：")
                    .to_owned()
            })
            .and_then(empty2none);

        let format = div_ele
            .next()
            .map(|ele| {
                ele.inner_html()
                    .trim()
                    .trim_start_matches("专辑类型：")
                    .to_owned()
            })
            .and_then(empty2none);

        let tags = fragment
            .select(&TAG)
            .map(|ele| ele.select(&A).next().unwrap().inner_html())
            .collect();

        let mut div_ele = fragment.select(&DESC);
        let description = div_ele
            .next()
            .map(|ele| ele.inner_html().trim().to_owned())
            .and_then(empty2none);

        let content = div_ele
            .next()
            .map(|ele| ele.inner_html().trim().to_owned())
            .and_then(empty2none);

        let tracks = fragment
            .select(&TRACK)
            .map(|ele| ele.inner_html().trim().to_owned())
            .collect();

        Album {
            title,
            cover,
            source,
            artists,
            companies,
            pub_time,
            genre,
            medium,
            code,
            format,
            tags,
            description,
            content,
            tracks,
        }
    }
}

fn empty2none(input: String) -> Option<String> {
    if input.is_empty() {
        None
    } else {
        Some(input)
    }
}

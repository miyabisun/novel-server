//! The page of the real binary in a real browser, for what only a browser's engine shows: the
//! layout of a 400px window (the `@media` rules), sizes and boxes, the colours the theme
//! resolves to, the focus ring the keyboard brings, the manifest the browser reads and the icon
//! it draws. The rest of the page is tested in jsdom (client/src/*.test.ts).
//!
//! One browser and one server: the tests run one after another, in the order they are written.
//! The server serves the page built in client/build; its API is answered in the page
//! (tests/support/api.js), so that nothing reaches a novel site. `cargo test --test page --
//! <words>` runs the tests whose titles hold any of the words; `--skip <words>` leaves those
//! out.
#![allow(clippy::unwrap_used)] // helpers outside #[test] fns
use serde_json::{json, Value};
use std::{
    future::Future,
    path::Path,
    pin::Pin,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

use browser::{free_port, Page, AGAIN, TRIES};

#[path = "support/browser.rs"]
mod browser;

type Run = Pin<Box<dyn Future<Output = ()> + Send>>;
/// A test: its title, and what it does on the page of the server.
type Test = (&'static str, fn(Arc<Page>, Arc<Server>) -> Run);

macro_rules! test {
    ($title:literal, $test:ident) => {
        ($title, |page, server| Box::pin($test(page, server)))
    };
}

/// What every document of the page runs first: the finders and measures of the tests, and the
/// stand-in for the server's API.
const PAGE_JS: &str = include_str!("support/page.js");
const API_JS: &str = include_str!("support/api.js");

/// The page surfaces (`--c-bg` in global.sass): Kinari is light, Washi e-paper, Sumi dark.
const KINARI: &str = "rgb(250, 246, 239)";
const WASHI: &str = "rgb(250, 250, 250)";
const SUMI: &str = "rgb(25, 25, 25)";

/// The real binary, serving client/build from a database in a directory of its own, on a port
/// nothing else held.
struct Server {
    /// Where its database lies, which goes with it.
    _dir: tempfile::TempDir,
    child: Child,
    port: u16,
}

impl Server {
    async fn start() -> Self {
        let root = env!("CARGO_MANIFEST_DIR");
        assert!(
            Path::new(root).join("client/build/index.html").is_file(),
            "the page is not built: run `npm run build` in client/ first"
        );
        let dir = tempfile::tempdir().unwrap();
        let log = dir.path().join("server.log");
        for _ in 0..TRIES {
            let port = free_port();
            let mut child = Command::new(env!("CARGO_BIN_EXE_novel-server"))
                .current_dir(root)
                .env("PORT", port.to_string())
                .env("DATABASE_PATH", dir.path().join("novel.db"))
                .env("BASE_PATH", "")
                .stdout(std::fs::File::create(&log).unwrap())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            // it says so once the port is its own; that the port answers is no sign, for
            // another run may have got the same one
            let deadline = Instant::now() + Duration::from_secs(20);
            let ended = loop {
                if std::fs::read_to_string(&log)
                    .unwrap_or_default()
                    .contains("Server running on")
                {
                    break false;
                }
                if child.try_wait().unwrap().is_some() {
                    break true;
                }
                assert!(Instant::now() < deadline, "the server did not come up");
                tokio::time::sleep(AGAIN).await;
            };
            if !ended {
                return Self {
                    _dir: dir,
                    child,
                    port,
                };
            }
        }
        panic!(
            "the server did not start: {}",
            std::fs::read_to_string(&log).unwrap_or_default()
        );
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---- finding, pressing and waiting on the page ----

fn number(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("{value} is not a number"))
}

/// The header, where the tabs and the menu are.
const HEADER: &str = "document.querySelector('header')";
/// The menu the hamburger opens.
const MENU: &str = "novel.one('navigation', 'アプリメニュー')";

/// Presses what `target` (a script's expression, given `args`) finds with the left button of
/// the mouse, once it shows and nothing lies over its middle: there, as a person would.
async fn click(page: &Page, target: &str, args: Value) {
    let at = page
        .until(
            &format!("{target} can be pressed"),
            &format!(
                "const element = {target}; return element && novel.shows(element) && novel.aim(element);"
            ),
            args,
        )
        .await;
    let (x, y) = (number(&at[0]), number(&at[1]));
    page.mouse("moved", x, y, "none").await;
    page.mouse("pressed", x, y, "left").await;
    page.mouse("released", x, y, "left").await;
}

/// Waits until what `target` finds shows.
async fn shows(page: &Page, target: &str, args: Value) {
    page.until(
        &format!("{target} shows"),
        &format!("return novel.shows({target});"),
        args,
    )
    .await;
}

/// Waits until what `target` finds is gone, or hidden.
async fn gone(page: &Page, target: &str, args: Value) {
    page.until(
        &format!("{target} is gone"),
        &format!("return !novel.shows({target});"),
        args,
    )
    .await;
}

/// Waits until what `script` returns is `expected`, and fails saying `what` it waited for and
/// what it returned last.
async fn until_is(page: &Page, what: &str, script: &str, args: Value, expected: Value) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let got = page.js(script, args.clone()).await;
        if got == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{what}: {got} after 5s, not {expected}"
        );
        tokio::time::sleep(AGAIN).await;
    }
}

/// Opens the ranking of narou from the favorites, as the tab does.
async fn open_ranking(page: &Page, server: &Server) {
    page.open(&server.url("/")).await;
    click(
        page,
        &format!("novel.one('link', 'nar', {HEADER})"),
        Value::Null,
    )
    .await;
    shows(page, "document.querySelector('.toolbar')", Value::Null).await;
}

/// In a browser tab theme-color carries the page background of the active theme.
async fn tab_theme_color(page: &Page, surface: &str) {
    until_is(
        page,
        "the theme-color of the tab",
        "return novel.themeColor();",
        Value::Null,
        json!(surface),
    )
    .await;
    until_is(
        page,
        "the background of the page",
        "return getComputedStyle(document.body).backgroundColor;",
        Value::Null,
        json!(surface),
    )
    .await;
}

/// Android Chrome 153-155 draws an installed app's system buttons over the window background,
/// fixed at install from the manifest `background_color` (#F8F9FA without one), and picks
/// their contrast from theme-color. The standalone theme-color must come first, so it wins
/// there, and carry the Sumi backdrop the manifest declares.
async fn installed_app_theme_color(page: &Page) {
    let first = page
        .js(
            "const meta = document.querySelector('meta[name=\"theme-color\"]'); \
             return [meta.media, novel.rgb(meta.content)];",
            Value::Null,
        )
        .await;
    assert_eq!(
        first,
        json!(["(display-mode: standalone)", SUMI]),
        "the first theme-color is the installed app's, on Sumi"
    );
}

/// The manifest the browser read for the page: its URL and what it declares.
async fn manifest_of(page: &Page) -> (String, Value) {
    let read = page.cdp("Page.getAppManifest", json!({})).await;
    assert_eq!(
        read["errors"],
        json!([]),
        "the manifest reads without errors"
    );
    let data: Value = serde_json::from_str(read["data"].as_str().unwrap()).unwrap();
    (read["url"].as_str().unwrap().to_owned(), data)
}

// ---- the tests: the page in a 400px window, as a phone shows it ----

async fn nav_at_400(page: Arc<Page>, server: Arc<Server>) {
    page.open(&server.url("/")).await;
    // compact brand dual: the root's short label is "novel"; the desktop label "favorite" hides
    shows(
        &page,
        &format!("novel.one('link', 'novel', {HEADER})"),
        Value::Null,
    )
    .await;
    gone(
        &page,
        &format!("novel.one('link', 'favorite', {HEADER})"),
        Value::Null,
    )
    .await;
    for name in ["nar", "kak", "noc"] {
        shows(
            &page,
            &format!("novel.one('link', arguments[0], {HEADER})"),
            json!(name),
        )
        .await;
    }
    shows(
        &page,
        &format!("novel.one('button', 'メニュー', {HEADER})"),
        Value::Null,
    )
    .await;
    gone(
        &page,
        &format!("novel.one('link', 'narou', {HEADER})"),
        Value::Null,
    )
    .await;
    assert_eq!(
        page.js("return novel.spills();", Value::Null).await,
        json!(false),
        "the page spills sideways"
    );
}

async fn toolbar(page: Arc<Page>, server: Arc<Server>) {
    // client-side navigation, as a person goes there
    open_ranking(&page, &server).await;
    let styles = page
        .js(
            "const style = getComputedStyle(document.querySelector('.toolbar')); \
             return [style.position, style.top, style.backgroundColor, Math.round(parseFloat(style.height))];",
            Value::Null,
        )
        .await;
    // --c-surface under Sumi (#232323) is an opaque rgb
    assert_eq!(
        styles,
        json!(["sticky", "48px", "rgb(35, 35, 35)", 40]),
        "the toolbar is a sticky band of 40px on the surface"
    );
    let select = page
        .js(
            "return Math.round(parseFloat(getComputedStyle(document.querySelector('.genre-select')).height));",
            Value::Null,
        )
        .await;
    assert_eq!(select, json!(36), "the genre select is 36px high");

    page.js("window.scrollTo(0, 200);", Value::Null).await;
    shows(&page, "document.querySelector('.toolbar')", Value::Null).await;
    let background = page
        .js(
            "return getComputedStyle(document.querySelector('.toolbar')).backgroundColor;",
            Value::Null,
        )
        .await;
    assert!(
        background != json!("rgba(0, 0, 0, 0)") && background != json!("transparent"),
        "the toolbar stays opaque when scrolled: {background}"
    );
}

async fn focus_rings(page: Arc<Page>, server: Arc<Server>) {
    open_ranking(&page, &server).await;
    for select in [".genre-select", ".period-select"] {
        shows(&page, "document.querySelector(arguments[0])", json!(select)).await;
    }
    for select in [".genre-select", ".period-select"] {
        // the keyboard brings the focus there, so that :focus-visible applies
        page.js(
            "document.querySelector(arguments[0]).focus();",
            json!(select),
        )
        .await;
        page.key("Shift+Tab").await;
        page.key("Tab").await;
        let ring = page
            .js(
                "const element = document.querySelector(arguments[0]); \
                 const style = getComputedStyle(element); \
                 const box = element.getBoundingClientRect(); \
                 const width = parseFloat(style.outlineWidth) || 0; \
                 const offset = parseFloat(style.outlineOffset) || 0; \
                 const band = document.querySelector('.toolbar').getBoundingClientRect(); \
                 const header = document.querySelector('header').getBoundingClientRect(); \
                 return { focusVisible: element.matches(':focus-visible'), style: style.outlineStyle, \
                     width, offset, top: box.top - width - offset, bottom: box.bottom + width + offset, \
                     floor: Math.max(band.top, header.bottom), band: band.bottom };",
                json!(select),
            )
            .await;
        assert_eq!(ring["focusVisible"], json!(true), "{select}: {ring}");
        assert_ne!(ring["style"], json!("none"), "{select}: {ring}");
        assert_eq!(ring["width"], json!(2), "{select}: {ring}");
        assert_eq!(ring["offset"], json!(-1), "{select}: {ring}");
        // strictly inside: the ring enters neither the app header nor leaves the band
        assert!(
            number(&ring["top"]) >= number(&ring["floor"]),
            "{select}: the ring reaches above the band: {ring}"
        );
        assert!(
            number(&ring["bottom"]) <= number(&ring["band"]),
            "{select}: the ring reaches below the band: {ring}"
        );
    }
}

async fn reader_menu(page: Arc<Page>, server: Arc<Server>) {
    page.open(&server.url("/")).await;
    click(&page, "novel.one('link', 'テスト小説')", Value::Null).await;
    shows(&page, "document.querySelector('.reader-bar')", Value::Null).await;
    gone(
        &page,
        "document.querySelector('.reader-bar .toc-btn')",
        Value::Null,
    )
    .await;
    gone(
        &page,
        "document.querySelector('.reader-bar .fav-btn-remove')",
        Value::Null,
    )
    .await;

    click(
        &page,
        &format!("novel.one('button', 'メニュー', {HEADER})"),
        Value::Null,
    )
    .await;
    until_is(
        &page,
        "the first action of the menu",
        &format!("return {MENU}?.querySelector('button.menu-item')?.textContent.trim() ?? null;"),
        Value::Null,
        json!("テーマ設定"),
    )
    .await;
    shows(
        &page,
        &format!("novel.one('button', '目次', {MENU})"),
        Value::Null,
    )
    .await;
    shows(
        &page,
        &format!("novel.one('button', 'お気に入りから削除', {MENU})"),
        Value::Null,
    )
    .await;
}

async fn theme_color(page: Arc<Page>, server: Arc<Server>) {
    page.look("light", 400, 800, true).await;
    page.open(&server.url("/")).await;
    tab_theme_color(&page, KINARI).await;
    installed_app_theme_color(&page).await;

    // 自動 follows the OS while the page stays open
    page.look("dark", 400, 800, true).await;
    tab_theme_color(&page, SUMI).await;

    click(
        &page,
        &format!("novel.one('button', 'メニュー', {HEADER})"),
        Value::Null,
    )
    .await;
    click(
        &page,
        &format!("novel.one('button', 'テーマ設定', {MENU})"),
        Value::Null,
    )
    .await;
    let radio = "novel.one('radio', arguments[0])";
    for (label, surface) in [
        ("ライト", KINARI),
        ("電子ペーパー", WASHI),
        ("ダーク", SUMI),
    ] {
        click(&page, radio, json!(label)).await;
        tab_theme_color(&page, surface).await;
        installed_app_theme_color(&page).await;
    }

    // a fixed choice ignores the OS switch; 自動 picks the OS back up
    page.look("light", 400, 800, true).await;
    tab_theme_color(&page, SUMI).await;
    click(&page, radio, json!("自動")).await;
    tab_theme_color(&page, KINARI).await;

    click(&page, radio, json!("電子ペーパー")).await;
    tab_theme_color(&page, WASHI).await;
    page.reload().await;
    tab_theme_color(&page, WASHI).await;
    installed_app_theme_color(&page).await;
}

async fn manifest(page: Arc<Page>, server: Arc<Server>) {
    page.open(&server.url("/")).await;
    let (_, manifest) = manifest_of(&page).await;
    assert_eq!(manifest["display"], json!("standalone"), "{manifest}");
    for field in ["background_color", "theme_color"] {
        let colour = page
            .js("return novel.rgb(arguments[0]);", manifest[field].clone())
            .await;
        assert_eq!(colour, json!(SUMI), "{field} of the manifest is Sumi");
    }
}

/// Android builds an adaptive home screen icon only from a maskable icon; an "any" icon
/// becomes a legacy icon that the launcher shrinks into a white circle. The maskable icon must
/// paint the whole square and keep its artwork inside the safe zone, large enough to read like
/// a native app icon.
async fn maskable_icon(page: Arc<Page>, server: Arc<Server>) {
    page.open(&server.url("/")).await;
    let (url, manifest) = manifest_of(&page).await;
    let icons = manifest["icons"].as_array().unwrap();
    let purposes = |icon: &Value| -> Vec<String> {
        icon["purpose"]
            .as_str()
            .unwrap_or("any")
            .split(' ')
            .map(str::to_owned)
            .collect()
    };
    assert!(
        icons
            .iter()
            .any(|icon| purposes(icon).contains(&"any".to_owned())),
        "an icon is for any purpose: {manifest}"
    );
    let maskable = icons
        .iter()
        .find(|icon| purposes(icon).contains(&"maskable".to_owned()))
        .unwrap_or_else(|| panic!("no icon is maskable: {manifest}"));

    let src = reqwest::Url::parse(&url)
        .unwrap()
        .join(maskable["src"].as_str().unwrap())
        .unwrap()
        .to_string();
    let answer = reqwest::get(&src).await.unwrap();
    let kind = answer.headers()["content-type"]
        .to_str()
        .unwrap()
        .to_owned();
    assert!(kind.contains("image/svg+xml"), "{src} is served as {kind}");

    let icon = page
        .js("return novel.maskable(arguments[0]);", json!(src))
        .await;
    assert_eq!(
        icon["opaque"],
        json!(true),
        "the icon paints its corners: {icon}"
    );
    assert_eq!(icon["outsideSafeZone"], json!(0), "{icon}");
    let radius = number(&icon["artworkRadius"]);
    assert!(
        (0.3..=0.4).contains(&radius),
        "the artwork reaches {radius} of the icon, not 0.3 to 0.4"
    );
}

/// The tests, in the order they run.
const TESTS: &[Test] = &[
    test!(
        "nav shows novel + 3-letter site labels and hamburger without overflow",
        nav_at_400
    ),
    test!(
        "ranking toolbar is opaque surface band with 36px controls",
        toolbar
    ),
    test!(
        "ranking select focus rings stay fully inside the toolbar band",
        focus_rings
    ),
    test!(
        "reader moves 目次 and unfav into hamburger on compact viewport",
        reader_menu
    ),
    test!(
        "theme-color follows every theme choice in a tab and stays Sumi in an installed app",
        theme_color
    ),
    test!(
        "installed app manifest declares the Sumi backdrop",
        manifest
    ),
    test!(
        "installed app offers a full-bleed maskable icon",
        maskable_icon
    ),
];

/// Which tests a run takes, as `cargo test` passes its words on: titles that hold any of the
/// words (all, when there are none), and none that hold a word after `--skip`; with
/// `--exact`, a word is a whole title. Other options are left alone.
fn chosen(args: &[String]) -> impl Fn(&str) -> bool {
    let mut words = Vec::new();
    let mut skipped = Vec::new();
    let mut exact = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--skip" => skipped.extend(args.next().cloned()),
            "--exact" => exact = true,
            option if option.starts_with('-') => {}
            word => words.push(word.to_owned()),
        }
    }
    move |title: &str| {
        let holds = |word: &String| {
            if exact {
                title == word
            } else {
                title.contains(word.as_str())
            }
        };
        (words.is_empty() || words.iter().any(holds)) && !skipped.iter().any(holds)
    }
}

/// What a panic said.
fn said(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|&said| said.to_owned()))
        .unwrap_or_default()
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let chosen = chosen(&args);
    let tests: Vec<&Test> = TESTS.iter().filter(|test| chosen(test.0)).collect();
    if args.iter().any(|arg| arg == "--list") {
        for (title, _) in &tests {
            println!("{title}: test");
        }
        return std::process::ExitCode::SUCCESS;
    }
    println!("\nrunning {} tests", tests.len());
    if tests.is_empty() {
        return std::process::ExitCode::SUCCESS;
    }
    let began = Instant::now();
    let server = Arc::new(Server::start().await);
    let page = Arc::new(Page::start(&[PAGE_JS, API_JS]).await);
    let mut failed = Vec::new();
    for (title, test) in &tests {
        // each test begins on a page of no state, in a phone's window of 400 by 800 in the dark
        page.open(&server.url("/")).await;
        page.js("localStorage.clear(); sessionStorage.clear();", Value::Null)
            .await;
        page.look("dark", 400, 800, true).await;
        let began = Instant::now();
        let ran = tokio::spawn(test(page.clone(), server.clone())).await;
        let took = began.elapsed().as_secs_f64();
        match ran {
            Ok(()) => println!("test {title} ... ok ({took:.2}s)"),
            Err(error) => {
                println!("test {title} ... FAILED: {}", said(&*error.into_panic()));
                failed.push(*title);
            }
        }
    }
    let page = Arc::try_unwrap(page).ok().unwrap();
    page.close().await;
    drop(server);
    let seconds = began.elapsed().as_secs_f64();
    let count = tests.len();
    if failed.is_empty() {
        println!("\ntest result: ok. {count} passed; 0 failed; finished in {seconds:.2}s\n");
        std::process::ExitCode::SUCCESS
    } else {
        println!("\nfailures:");
        for title in &failed {
            println!("    {title}");
        }
        println!(
            "\ntest result: FAILED. {} passed; {} failed; finished in {seconds:.2}s\n",
            count - failed.len(),
            failed.len()
        );
        std::process::ExitCode::from(101)
    }
}

#![cfg(feature = "ssr")]
#![recursion_limit = "256"]

//! The document shell as the server writes it: the pilot notice is always in
//! the tree so both sides hydrate the same markup, and the shell marks the
//! document when the notice is meant to show. The OnceLock behind the flag is
//! set once per process, so this file holds the one test that sets it.

use leptos::prelude::*;
use leptos::tachys::view::Position;
use leptos_router::location::RequestUrl;
use web::app::{App, shell};

fn render_shell(url: &str) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new(url));
        let options = LeptosOptions::builder()
            .output_name("dac2")
            .site_root("target/site")
            .build();
        let view = shell(options);
        let mut html = String::new();
        view.to_html_with_buf(
            &mut html,
            &mut Position::FirstChild,
            true,
            false,
            Vec::new(),
        );
        html
    })
}

fn render_app(url: &str) -> String {
    Owner::new().with(move || {
        provide_context(RequestUrl::new(url));
        let view = view! { <App/> };
        let mut html = String::new();
        view.to_html_with_buf(
            &mut html,
            &mut Position::FirstChild,
            true,
            false,
            Vec::new(),
        );
        html
    })
}

#[test]
fn the_pilot_notice_is_in_every_page_and_the_shell_switches_it_on() {
    let home = render_app("/");
    let notice = home
        .find("นี่คือสำเนาทดลอง ข้อมูลที่กรอกใช้เพื่อการศึกษาและอาจถูกลบ")
        .expect("the notice is rendered on the home page");
    let term = home.find("(pilot)").expect("the formal term follows");
    assert!(notice < term, "plain wording leads the formal term");
    let header = home
        .find("site-header")
        .expect("the header follows the notice");
    assert!(notice < header);
    assert!(
        render_app("/login").contains("class=\"pilot-notice\""),
        "the notice is part of the shared shell, not one page"
    );

    // Off by default: the attribute is absent and the stylesheet keeps the
    // notice hidden.
    let plain_shell = render_shell("/");
    assert!(plain_shell.contains("<html lang=\"th\">"), "{plain_shell}");
    assert!(!plain_shell.contains("data-pilot"));

    web::settings::set_pilot_notice(true);
    let pilot_shell = render_shell("/");
    assert!(
        pilot_shell.contains("<html lang=\"th\" data-pilot=\"true\">"),
        "{pilot_shell}"
    );
}

// The entry screen, as DESIGN.md "Identity and the entry screen" lays it out:
// the root address is the login form, it carries the mascot, the wordmark,
// the one-line tagline, its own pilot line and the cohort byline, and the
// landing card with its demo link is gone. The shared strip and header are
// still in the tree - the stylesheet steps them aside on this screen - so
// both sides hydrate the same markup.
#[test]
fn the_root_address_is_the_entry_screen() {
    let root = render_app("/");
    for expected in [
        "entry-screen",
        "/takai-bust.webp",
        "ตาไก๊",
        "คิดกำไรขาดทุนสวนทุเรียน ให้รู้ก่อนขาย",
        "name=\"email\"",
        "name=\"password\"",
        "ลืมรหัสผ่าน",
        "สมัครใช้งาน",
        "สำเนาทดลอง · ข้อมูลที่กรอกใช้เพื่อการศึกษาและอาจถูกลบ",
        "จากชาว Durian Academy รุ่นที่ 2",
    ] {
        assert!(root.contains(expected), "missing {expected} in {root}");
    }
    for gone in [
        "เห็นรายได้ ต้นทุน และกำไรของสวนในฤดูกาลเดียว",
        "href=\"/demo\"",
        "ใช้อีเมลเดียวกับที่สมัครบัญชี",
        "รหัสผ่านที่ตั้งไว้ตอนสมัคร",
        "DAC2",
    ] {
        assert!(
            !root.contains(gone),
            "{gone} should be gone from the entry screen"
        );
    }
    // The old login address renders the same screen; only the header link's
    // aria-current differs, because one address is the header's own.
    let login = render_app("/login");
    for expected in [
        "entry-screen",
        "/takai-bust.webp",
        "คิดกำไรขาดทุนสวนทุเรียน ให้รู้ก่อนขาย",
    ] {
        assert!(login.contains(expected), "missing {expected} on /login");
    }
    assert!(
        !render_app("/register").contains("entry-screen"),
        "only the entry addresses hide the shared header"
    );
}

#[test]
fn the_shell_names_the_application_and_its_mark() {
    let shell = render_shell("/");
    assert!(shell.contains("<title>บัญชีตาไก๊ · Takai</title>"), "{shell}");
    assert!(
        shell.contains("rel=\"icon\" href=\"/takai-mark.svg\""),
        "{shell}"
    );
    assert!(
        shell.contains("rel=\"apple-touch-icon\" href=\"/takai-icon-180.png\""),
        "{shell}"
    );
}

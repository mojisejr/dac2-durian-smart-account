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

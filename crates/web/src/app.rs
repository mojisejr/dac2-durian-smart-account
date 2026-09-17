use leptos::{form::ActionForm, prelude::*};
use leptos_router::{
    components::{A, Route, Router, Routes},
    hooks::{use_location, use_query_map},
    path,
};

use crate::assets::AssetPage;
use crate::auth::{
    Login, Register, RequestPasswordReset, ResendVerification, ResetPassword, VerifyEmail,
};
use crate::plan_ui::{
    ActualClosePage, ActualComparisonPage, ActualReviewPage, DemoPage, NewSeasonPage, PlanHubPage,
    PlanSectionRoute, PlansPage, QuickPlanRoute, SeasonHistoryPage,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <AppShell/>
        </Router>
    }
}

/// The entry screen carries the identity itself - the mascot, the wordmark,
/// the one-line pilot notice - so the shared header and the strip above it
/// step aside there and nowhere else.
pub fn entry_path(path: &str) -> bool {
    matches!(path.trim_end_matches('/'), "" | "/login")
}

#[component]
fn AppShell() -> impl IntoView {
    let location = use_location();
    let entry = move || location.pathname.with(|path| entry_path(path));
    view! {
        <main class:entry-screen=entry>
            // Rendered on every page and shown only when the shell marks
            // the document as a pilot copy, so the same markup hydrates
            // on localhost and on the study address alike.
            <p class="pilot-notice" role="note">
                "นี่คือสำเนาทดลอง ข้อมูลที่กรอกใช้เพื่อการศึกษาและอาจถูกลบ"
                <span class="pilot-notice-term">" (pilot)"</span>
            </p>
            <header class="site-header">
                <A href="/"><Mark/>"ตาไก๊"</A>
            </header>
            <Routes fallback=NotFound>
                <Route path=path!("") view=EntryPage/>
                <Route path=path!("login") view=EntryPage/>
                <Route path=path!("register") view=RegisterPage/>
                <Route path=path!("verify-email") view=VerifyEmailPage/>
                <Route path=path!("forgot-password") view=ForgotPasswordPage/>
                <Route path=path!("reset-password") view=ResetPasswordPage/>
                <Route path=path!("demo") view=DemoPage/>
                <Route path=path!("history") view=SeasonHistoryPage/>
                <Route path=path!("plans") view=PlansPage/>
                <Route path=path!("plans/new") view=NewSeasonPage/>
                <Route path=path!("plans/:id") view=PlanHubPage/>
                <Route path=path!("plans/:id/quick/:step") view=QuickPlanRoute/>
                <Route path=path!("plans/:id/close") view=ActualClosePage/>
                <Route path=path!("plans/:id/close/review") view=ActualReviewPage/>
                <Route path=path!("plans/:id/comparison") view=ActualComparisonPage/>
                <Route path=path!("plans/:id/assets") view=AssetPage/>
                <Route path=path!("plans/:id/:section") view=PlanSectionRoute/>
            </Routes>
        </main>
    }
}

/// ตาไก๊ himself, hat to collar, in a 24px circle beside his name. The owner
/// chose the face over the hat silhouette for every mark on 2026-09-17; the
/// favicon and the home-screen icon are the same cut on a cream disc in
/// `public/takai-icon-*.png`.
#[component]
fn Mark() -> impl IntoView {
    view! {
        <img class="takai-mark" src="/takai-mark-48.png" width="24" height="24" alt="" aria-hidden="true"/>
    }
}

#[component]
fn RegisterPage() -> impl IntoView {
    let register = ServerAction::<Register>::new();
    let resend = ServerAction::<ResendVerification>::new();

    view! {
        <section class="card auth-card">
            <p class="eyebrow">"สร้างบัญชี"</p>
            <h1>"สมัครใช้งาน"</h1>
            <p>"กรอกอีเมลและตั้งรหัสผ่าน จากนั้นเปิดลิงก์ยืนยันที่ส่งไปทางอีเมล"</p>
            <ActionForm action=register>
                <FormField id="register-email" label="อีเมล" name="email" input_type="email" autocomplete="email" hint="ใช้รับลิงก์ยืนยันและกู้รหัสผ่าน"/>
                <PasswordField
                    id="register-password"
                    label="รหัสผ่าน"
                    name="password"
                    autocomplete="new-password"
                    minlength="6"
                    hint="ใช้อย่างน้อย 6 ตัวอักษร"
                />
                <PasswordField
                    id="register-password-confirm"
                    label="ยืนยันรหัสผ่าน"
                    name="password_confirm"
                    autocomplete="new-password"
                    minlength="6"
                    hint="พิมพ์รหัสผ่านเดิมอีกครั้ง"
                />
                <button class="primary" type="submit">"สมัครใช้งาน"</button>
            </ActionForm>
            <ActionMessage action=register/>

            <details>
                <summary>"ยังไม่ได้รับลิงก์ยืนยัน"</summary>
                <ActionForm action=resend>
                    <FormField id="resend-email" label="อีเมลที่ใช้สมัคร" name="email" input_type="email" autocomplete="email" hint="ระบบจะส่งลิงก์ยืนยันฉบับใหม่ ถ้ามีบัญชีนี้"/>
                    <button class="secondary" type="submit">"ส่งลิงก์ใหม่"</button>
                </ActionForm>
                <ActionMessage action=resend/>
            </details>

            <p class="alternate">"มีบัญชีแล้ว? " <A href="/login">"เข้าสู่ระบบ"</A></p>
        </section>
    }
}

#[component]
fn EntryPage() -> impl IntoView {
    let action = ServerAction::<Login>::new();
    let query = use_query_map();
    let verified = move || query.with(|params| params.get("verified").as_deref() == Some("1"));
    view! {
        <section class="card auth-card entry-card">
            <div class="mascot" aria-hidden="true">
                <picture>
                    <source srcset="/takai-bust.webp" type="image/webp"/>
                    <img src="/takai-bust.png" width="256" height="256" alt=""/>
                </picture>
            </div>
            <h1 class="wordmark">"ตาไก๊"</h1>
            <p class="tagline">"คิดกำไรขาดทุนสวนทุเรียน ให้รู้ก่อนขาย"</p>
            <Show when=verified>
                <p class="form-message">"ยืนยันอีเมลเรียบร้อยแล้ว เข้าสู่ระบบได้เลย"</p>
            </Show>
            <ActionForm action=action>
                <FormField id="login-email" label="อีเมล" name="email" input_type="email" autocomplete="email"/>
                <PasswordField
                    id="login-password"
                    label="รหัสผ่าน"
                    name="password"
                    autocomplete="current-password"
                />
                <button class="primary" type="submit">"เข้าสู่ระบบ"</button>
            </ActionForm>
            <ActionMessage action=action/>
            <p class="alternate"><A href="/forgot-password">"ลืมรหัสผ่าน"</A></p>
            <p class="alternate">"ยังไม่มีบัญชี? " <A href="/register">"สมัครใช้งาน"</A></p>
        </section>
        <p class="entry-pilot" role="note">"สำเนาทดลอง · ข้อมูลที่กรอกใช้เพื่อการศึกษาและอาจถูกลบ"</p>
        <p class="byline">"จากชาว Durian Academy รุ่นที่ 2"</p>
    }
}

#[component]
fn VerifyEmailPage() -> impl IntoView {
    let action = ServerAction::<VerifyEmail>::new();
    let query = use_query_map();
    let token = move || query.with(|params| params.get("token")).unwrap_or_default();
    let invalid = move || query.with(|params| params.get("invalid").as_deref() == Some("1"));

    view! {
        <section class="card auth-card">
            <p class="eyebrow">"ยืนยันอีเมล"</p>
            <h1>"เปิดใช้งานบัญชี"</h1>
            <Show when=invalid>
                <p class="form-message">"ลิงก์นี้ใช้ไม่ได้ อาจหมดอายุหรือเคยใช้แล้ว กลับไปหน้าสมัครเพื่อขอลิงก์ใหม่ได้"</p>
            </Show>
            <p>"กดปุ่มด้านล่างเพื่อใช้ลิงก์ยืนยันนี้หนึ่งครั้ง"</p>
            <ActionForm action=action>
                <input type="hidden" name="token" value=token/>
                <button class="primary" type="submit">"ยืนยันอีเมล"</button>
            </ActionForm>
            <ActionMessage action=action/>
            <p class="alternate"><A href="/login">"กลับไปเข้าสู่ระบบ"</A></p>
        </section>
    }
}

#[component]
fn ForgotPasswordPage() -> impl IntoView {
    let action = ServerAction::<RequestPasswordReset>::new();
    view! {
        <section class="card auth-card">
            <p class="eyebrow">"กู้บัญชี"</p>
            <h1>"ลืมรหัสผ่าน"</h1>
            <p>"กรอกอีเมลที่ใช้สมัคร ระบบจะส่งลิงก์ตั้งรหัสผ่านใหม่ให้หากมีบัญชีนี้"</p>
            <ActionForm action=action>
                <FormField id="forgot-email" label="อีเมลที่ใช้สมัคร" name="email" input_type="email" autocomplete="email" hint="เพื่อความเป็นส่วนตัว ระบบจะแสดงคำตอบเหมือนกันไม่ว่าจะพบบัญชีหรือไม่"/>
                <button class="primary" type="submit">"ขอลิงก์ตั้งรหัสผ่านใหม่"</button>
            </ActionForm>
            <ActionMessage action=action/>
            <p class="alternate"><A href="/login">"กลับไปเข้าสู่ระบบ"</A></p>
        </section>
    }
}

#[component]
fn ResetPasswordPage() -> impl IntoView {
    let action = ServerAction::<ResetPassword>::new();
    let query = use_query_map();
    let token = move || query.with(|params| params.get("token")).unwrap_or_default();

    view! {
        <section class="card auth-card">
            <p class="eyebrow">"กู้บัญชี"</p>
            <h1>"ตั้งรหัสผ่านใหม่"</h1>
            <ActionForm action=action>
                <input type="hidden" name="token" value=token/>
                <PasswordField
                    id="reset-password"
                    label="รหัสผ่านใหม่"
                    name="new_password"
                    autocomplete="new-password"
                    minlength="6"
                    hint="ใช้อย่างน้อย 6 ตัวอักษร แล้วเข้าสู่ระบบด้วยรหัสใหม่นี้"
                />
                <PasswordField
                    id="reset-password-confirm"
                    label="ยืนยันรหัสผ่านใหม่"
                    name="new_password_confirm"
                    autocomplete="new-password"
                    minlength="6"
                    hint="พิมพ์รหัสผ่านใหม่อีกครั้ง"
                />
                <button class="primary" type="submit">"บันทึกรหัสผ่านใหม่"</button>
            </ActionForm>
            <ActionMessage action=action/>
            <p class="alternate"><A href="/login">"กลับไปเข้าสู่ระบบ"</A></p>
        </section>
    }
}

#[component]
fn FormField(
    id: &'static str,
    label: &'static str,
    name: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    /// Persistent help for a question the owner may not understand. A field a
    /// phone-fluent person already knows - a login email, a password - carries
    /// none, as DESIGN.md rules.
    #[prop(optional)]
    hint: Option<&'static str>,
    #[prop(default = "1")] minlength: &'static str,
) -> impl IntoView {
    let help_id = hint.map(|_| format!("{id}-help"));
    view! {
        <label>
            <span>{label}</span>
            <input
                id=id
                type=input_type
                name=name
                autocomplete=autocomplete
                minlength=minlength
                maxlength="1024"
                aria-describedby=help_id.clone()
                required
            />
            {hint.map(|hint| view! { <small id=help_id class="field-hint">{hint}</small> })}
        </label>
    }
}

/// A password field with a show/hide button beside it. The button is a
/// control of its own, 56 square to match the input and never a glyph inside
/// it, and it says
/// its state through `aria-pressed` and a Thai label rather than colour.
/// It works once the page hydrates; before that it is inert and the form
/// still submits with the password hidden.
#[component]
fn PasswordField(
    id: &'static str,
    label: &'static str,
    name: &'static str,
    autocomplete: &'static str,
    #[prop(optional)] hint: Option<&'static str>,
    #[prop(default = "1")] minlength: &'static str,
) -> impl IntoView {
    let help_id = hint.map(|_| format!("{id}-help"));
    let shown = RwSignal::new(false);
    let input_type = move || if shown.get() { "text" } else { "password" };
    let button_label = move || {
        if shown.get() {
            "ซ่อนรหัสผ่าน"
        } else {
            "แสดงรหัสผ่าน"
        }
    };
    view! {
        <div class="password-field">
            <label for=id>{label}</label>
            <div class="password-row">
                <input
                    id=id
                    type=input_type
                    name=name
                    autocomplete=autocomplete
                    minlength=minlength
                    maxlength="1024"
                    aria-describedby=help_id.clone()
                    required
                />
                <button
                    class="icon-button reveal"
                    type="button"
                    aria-pressed=move || shown.get().to_string()
                    aria-label=button_label
                    title=button_label
                    on:click=move |_| shown.update(|value| *value = !*value)
                >
                    <Show when=move || shown.get() fallback=EyeOpen>
                        <EyeClosed/>
                    </Show>
                </button>
            </div>
            {hint.map(|hint| view! { <small id=help_id class="field-hint">{hint}</small> })}
        </div>
    }
}

/// The eye, drawn in the same hand as the hat mark: one stroke, no fill.
#[component]
fn EyeOpen() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6-10-6-10-6z"/>
            <circle cx="12" cy="12" r="3"/>
        </svg>
    }
}

/// The same eye with a stroke across it.
#[component]
fn EyeClosed() -> impl IntoView {
    view! {
        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6-10-6-10-6z"/>
            <circle cx="12" cy="12" r="3"/>
            <path d="M4 4l16 16"/>
        </svg>
    }
}

#[component]
fn ActionMessage<S>(action: ServerAction<S>) -> impl IntoView
where
    S: server_fn::ServerFn<Output = String, Error = ServerFnError> + Clone + Send + Sync + 'static,
{
    view! {
        <p class="form-message" aria-live="polite">
            {move || action.value().get().map(plain_message)}
        </p>
    }
}

/// The sentence a form shows for what the server answered. A message the
/// server wrote for the owner is shown as written; the transport's own
/// prefix ("error running server function") is not the owner's language and
/// is dropped. Any other failure keeps its description, because it is the
/// only clue anyone has.
fn plain_message(result: Result<String, ServerFnError>) -> String {
    match result {
        Ok(message) => message,
        Err(ServerFnError::ServerError(message)) => message,
        Err(other) => other.to_string(),
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="card recovery-state">
            <h1>"ไม่พบหน้านี้"</h1>
            <p>"ลิงก์นี้ไม่มีอยู่ หรือหน้านี้ถูกย้ายไปแล้ว"</p>
            <A attr:class="button secondary" href="/">"กลับหน้าแรก"</A>
        </section>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    // The attribute lives on <html>, outside what the body hydrates, so a
    // notice that is on for the server is on for the client without either
    // side rendering a different tree.
    let pilot = crate::settings::pilot_notice().then_some("true");
    view! {
        <!DOCTYPE html>
        <html lang="th" data-pilot=pilot>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"บัญชีตาไก๊ · Takai"</title>
                <link rel="icon" href="/takai-icon-32.png" sizes="32x32" type="image/png"/>
                <link rel="icon" href="/takai-icon-64.png" sizes="64x64" type="image/png"/>
                <link rel="apple-touch-icon" href="/takai-icon-180.png"/>
                <link rel="stylesheet" href="/pkg/dac2.css"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
            </head>
            <body><App/></body>
        </html>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_server_message_is_shown_as_written_without_the_transport_prefix() {
        assert_eq!(
            plain_message(Err(ServerFnError::new(crate::auth::LOGIN_FAILED))),
            crate::auth::LOGIN_FAILED
        );
        assert_eq!(
            plain_message(Err(ServerFnError::new(
                "ลองหลายครั้งติดกันแล้ว กรุณารอ 5 นาที แล้วลองใหม่อีกครั้ง"
            ))),
            "ลองหลายครั้งติดกันแล้ว กรุณารอ 5 นาที แล้วลองใหม่อีกครั้ง"
        );
        assert_eq!(plain_message(Ok("เข้าสู่ระบบแล้ว".into())), "เข้าสู่ระบบแล้ว");
        assert!(plain_message(Err(ServerFnError::Request("offline".into()))).contains("offline"));
    }
}

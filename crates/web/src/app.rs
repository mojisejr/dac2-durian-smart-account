use leptos::{form::ActionForm, prelude::*};
use leptos_router::{
    components::{A, Route, Router, Routes},
    hooks::use_query_map,
    path,
};

use crate::auth::{
    Login, Register, RequestPasswordReset, ResendVerification, ResetPassword, VerifyEmail,
};
use crate::plan_ui::{
    ActualClosePage, ActualComparisonPage, ActualReviewPage, DemoPage, NewSeasonPage, PlanHubPage,
    PlanSectionRoute, PlansPage, QuickPlanRoute,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <header class="site-header">
                    <A href="/">"DAC2 — Durian Smart Account"</A>
                </header>
                <Routes fallback=NotFound>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("login") view=LoginPage/>
                    <Route path=path!("register") view=RegisterPage/>
                    <Route path=path!("verify-email") view=VerifyEmailPage/>
                    <Route path=path!("forgot-password") view=ForgotPasswordPage/>
                    <Route path=path!("reset-password") view=ResetPasswordPage/>
                    <Route path=path!("demo") view=DemoPage/>
                    <Route path=path!("plans") view=PlansPage/>
                    <Route path=path!("plans/new") view=NewSeasonPage/>
                    <Route path=path!("plans/:id") view=PlanHubPage/>
                    <Route path=path!("plans/:id/quick/:step") view=QuickPlanRoute/>
                    <Route path=path!("plans/:id/close") view=ActualClosePage/>
                    <Route path=path!("plans/:id/close/review") view=ActualReviewPage/>
                    <Route path=path!("plans/:id/comparison") view=ActualComparisonPage/>
                    <Route path=path!("plans/:id/:section") view=PlanSectionRoute/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section class="card">
            <p class="eyebrow">"DAC2"</p>
            <h1>"เริ่มใช้งาน"</h1>
            <p>"สมัครด้วยอีเมลและรหัสผ่าน หรือเข้าสู่ระบบ"</p>
            <div class="actions">
                <A attr:class="button primary" href="/register">"สมัครใช้งาน"</A>
                <A attr:class="button secondary" href="/login">"เข้าสู่ระบบ"</A>
            </div>
        </section>
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
            <p>"กรอกเพียงอีเมลและรหัสผ่าน แล้วเปิด Mailpit เพื่อยืนยันอีเมล"</p>
            <ActionForm action=register>
                <FormField label="อีเมล" name="email" input_type="email" autocomplete="email"/>
                <FormField
                    label="รหัสผ่าน (อย่างน้อย 15 ตัวอักษร)"
                    name="password"
                    input_type="password"
                    autocomplete="new-password"
                    minlength="15"
                />
                <button class="primary" type="submit">"สมัครใช้งาน"</button>
            </ActionForm>
            <ActionMessage action=register/>

            <details>
                <summary>"ยังไม่ได้รับลิงก์ยืนยัน"</summary>
                <ActionForm action=resend>
                    <FormField label="อีเมล" name="email" input_type="email" autocomplete="email"/>
                    <button class="secondary" type="submit">"ส่งลิงก์ใหม่"</button>
                </ActionForm>
                <ActionMessage action=resend/>
            </details>

            <p class="alternate">"มีบัญชีแล้ว? " <A href="/login">"เข้าสู่ระบบ"</A></p>
        </section>
    }
}

#[component]
fn LoginPage() -> impl IntoView {
    let action = ServerAction::<Login>::new();
    let query = use_query_map();
    let verified = move || query.with(|params| params.get("verified").as_deref() == Some("1"));
    view! {
        <section class="card auth-card">
            <p class="eyebrow">"ยินดีต้อนรับกลับ"</p>
            <h1>"เข้าสู่ระบบ"</h1>
            <Show when=verified>
                <p class="form-message">"ยืนยันอีเมลเรียบร้อยแล้ว เข้าสู่ระบบได้เลย"</p>
            </Show>
            <ActionForm action=action>
                <FormField label="อีเมล" name="email" input_type="email" autocomplete="email"/>
                <FormField
                    label="รหัสผ่าน"
                    name="password"
                    input_type="password"
                    autocomplete="current-password"
                />
                <button class="primary" type="submit">"เข้าสู่ระบบ"</button>
            </ActionForm>
            <ActionMessage action=action/>
            <p class="alternate"><A href="/forgot-password">"ลืมรหัสผ่าน"</A></p>
            <p class="alternate">"ยังไม่มีบัญชี? " <A href="/register">"สมัครใช้งาน"</A></p>
        </section>
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
                <p class="form-message">"ลิงก์ยืนยันไม่ถูกต้อง หมดอายุ หรือถูกใช้ไปแล้ว"</p>
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
            <p>"ระบบจะส่งลิงก์ตั้งรหัสผ่านใหม่ไปยัง Mailpit หากมีบัญชีนี้"</p>
            <ActionForm action=action>
                <FormField label="อีเมล" name="email" input_type="email" autocomplete="email"/>
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
                <FormField
                    label="รหัสผ่านใหม่ (อย่างน้อย 15 ตัวอักษร)"
                    name="new_password"
                    input_type="password"
                    autocomplete="new-password"
                    minlength="15"
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
    label: &'static str,
    name: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    #[prop(default = "1")] minlength: &'static str,
) -> impl IntoView {
    view! {
        <label>
            <span>{label}</span>
            <input
                type=input_type
                name=name
                autocomplete=autocomplete
                minlength=minlength
                maxlength="1024"
                required
            />
        </label>
    }
}

#[component]
fn ActionMessage<S>(action: ServerAction<S>) -> impl IntoView
where
    S: server_fn::ServerFn<Output = String> + Clone + Send + Sync + 'static,
    S::Error: Clone + std::fmt::Display + Send + Sync + 'static,
{
    view! {
        <p class="form-message" aria-live="polite">
            {move || {
                action
                    .value()
                    .get()
                    .map(|result| result.unwrap_or_else(|error| error.to_string()))
            }}
        </p>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="card">
            <h1>"ไม่พบหน้านี้"</h1>
            <A href="/">"กลับหน้าแรก"</A>
        </section>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="th">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="stylesheet" href="/pkg/dac2.css"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
            </head>
            <body><App/></body>
        </html>
    }
}

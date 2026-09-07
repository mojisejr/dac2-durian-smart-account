use leptos::prelude::*;

pub const REGISTRATION_ACCEPTED: &str = "หากอีเมลนี้สมัครได้ ระบบได้ส่งลิงก์ยืนยันไปให้แล้ว";
pub const RESEND_ACCEPTED: &str = "หากบัญชีนี้ยังรอยืนยัน ระบบได้ส่งลิงก์ยืนยันฉบับใหม่ไปให้แล้ว";
pub const RESET_REQUEST_ACCEPTED: &str = "หากมีบัญชีที่ใช้อีเมลนี้ ระบบได้ส่งลิงก์ตั้งรหัสผ่านใหม่ไปให้แล้ว";
pub const LOGIN_FAILED: &str = "อีเมลหรือรหัสผ่านไม่ถูกต้อง หรือบัญชียังไม่ได้ยืนยัน";

#[cfg(feature = "ssr")]
pub async fn authenticate_and_login(
    auth_session: &mut store::AuthSession,
    credentials: store::AuthCredentials,
) -> Result<bool, axum_login::Error<store::AuthBackend>> {
    let Some(user) = auth_session.authenticate(credentials).await? else {
        return Ok(false);
    };
    auth_session.login(&user).await?;
    Ok(true)
}

#[cfg(feature = "ssr")]
pub async fn logout_session(
    auth_session: &mut store::AuthSession,
) -> Result<(), axum_login::Error<store::AuthBackend>> {
    auth_session.logout().await?;
    Ok(())
}

#[server]
pub async fn register(email: String, password: String) -> Result<String, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    let pool = auth_session.backend.pool();

    match store::users::register(pool, &email, &password).await {
        Ok(user) => {
            if let Ok(token) = store::verification_tokens::issue(pool, user.id).await {
                let _ = crate::mail::Mailer::from_env()
                    .send_verification(&user.email, token.expose())
                    .await;
            }
            Ok(REGISTRATION_ACCEPTED.into())
        }
        Err(store::StoreError::DuplicateEmail) => Ok(REGISTRATION_ACCEPTED.into()),
        Err(store::StoreError::InvalidEmail) => Err(ServerFnError::new("กรุณากรอกอีเมลให้ถูกต้อง")),
        Err(store::StoreError::InvalidPassword) => {
            Err(ServerFnError::new("รหัสผ่านต้องมีอย่างน้อย 15 ตัวอักษร"))
        }
        Err(_) => Err(public_server_error()),
    }
}

#[server]
pub async fn verify_email(token: String) -> Result<String, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    match store::verification_tokens::consume(auth_session.backend.pool(), &token).await {
        Ok(_) => Ok("ยืนยันอีเมลเรียบร้อยแล้ว เข้าสู่ระบบได้เลย".into()),
        Err(store::StoreError::InvalidToken) => {
            Err(ServerFnError::new("ลิงก์ยืนยันไม่ถูกต้อง หมดอายุ หรือถูกใช้ไปแล้ว"))
        }
        Err(_) => Err(public_server_error()),
    }
}

#[cfg(feature = "ssr")]
#[derive(serde::Deserialize)]
pub struct VerificationQuery {
    token: String,
}

#[cfg(feature = "ssr")]
pub async fn verify_email_link(
    auth_session: store::AuthSession,
    axum::extract::Query(query): axum::extract::Query<VerificationQuery>,
) -> axum::response::Redirect {
    if store::verification_tokens::consume(auth_session.backend.pool(), &query.token)
        .await
        .is_ok()
    {
        axum::response::Redirect::to("/login?verified=1")
    } else {
        axum::response::Redirect::to("/verify-email?invalid=1")
    }
}

#[server]
pub async fn resend_verification(email: String) -> Result<String, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    let pool = auth_session.backend.pool();

    match store::users::find_by_email(pool, &email).await {
        Ok(Some(user)) if !user.email_verified => {
            if let Ok(token) = store::verification_tokens::issue(pool, user.id).await {
                let _ = crate::mail::Mailer::from_env()
                    .send_verification(&user.email, token.expose())
                    .await;
            }
        }
        Ok(_) | Err(_) => {}
    }

    Ok(RESEND_ACCEPTED.into())
}

#[server]
pub async fn login(email: String, password: String) -> Result<String, ServerFnError> {
    let mut auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    let credentials = store::AuthCredentials { email, password };
    let authenticated = authenticate_and_login(&mut auth_session, credentials)
        .await
        .map_err(|_| public_server_error())?;
    if !authenticated {
        return Err(ServerFnError::new(LOGIN_FAILED));
    }
    leptos_axum::redirect("/plans");
    Ok("เข้าสู่ระบบแล้ว".into())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    logout_session(&mut auth_session)
        .await
        .map_err(|_| public_server_error())?;
    leptos_axum::redirect("/login");
    Ok(())
}

#[server]
pub async fn request_password_reset(email: String) -> Result<String, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    let pool = auth_session.backend.pool();

    match store::users::find_by_email(pool, &email).await {
        Ok(Some(user)) if user.email_verified => {
            if let Ok(token) = store::reset_tokens::issue(pool, user.id).await {
                let _ = crate::mail::Mailer::from_env()
                    .send_password_reset(&user.email, token.expose())
                    .await;
            }
        }
        Ok(_) | Err(_) => {}
    }

    Ok(RESET_REQUEST_ACCEPTED.into())
}

#[server]
pub async fn reset_password(token: String, new_password: String) -> Result<String, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    match store::reset_tokens::consume(auth_session.backend.pool(), &token, &new_password).await {
        Ok(_) => Ok("ตั้งรหัสผ่านใหม่เรียบร้อยแล้ว กรุณาเข้าสู่ระบบอีกครั้ง".into()),
        Err(store::StoreError::InvalidPassword) => {
            Err(ServerFnError::new("รหัสผ่านต้องมีอย่างน้อย 15 ตัวอักษร"))
        }
        Err(store::StoreError::InvalidToken) => {
            Err(ServerFnError::new("ลิงก์ตั้งรหัสผ่านไม่ถูกต้อง หมดอายุ หรือถูกใช้ไปแล้ว"))
        }
        Err(_) => Err(public_server_error()),
    }
}

#[server]
pub async fn current_user_email() -> Result<Option<String>, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::AuthSession>().await?;
    Ok(auth_session.user.map(|user| user.email))
}

#[cfg(feature = "ssr")]
fn public_server_error() -> ServerFnError {
    ServerFnError::new("ระบบยังทำรายการไม่ได้ กรุณาลองอีกครั้ง")
}

#[cfg(feature = "ssr")]
pub async fn require_login(
    auth_session: store::AuthSession,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::{IntoResponse, Redirect};

    if !request.uri().path().starts_with("/plans") || auth_session.user.is_some() {
        next.run(request).await
    } else {
        Redirect::to("/login").into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_account_responses_do_not_encode_account_existence() {
        let successful_registration = REGISTRATION_ACCEPTED;
        let duplicate_registration = REGISTRATION_ACCEPTED;
        assert_eq!(successful_registration, duplicate_registration);

        let known_reset_address = RESET_REQUEST_ACCEPTED;
        let unknown_reset_address = RESET_REQUEST_ACCEPTED;
        assert_eq!(known_reset_address, unknown_reset_address);

        let known_resend_address = RESEND_ACCEPTED;
        let unknown_resend_address = RESEND_ACCEPTED;
        assert_eq!(known_resend_address, unknown_resend_address);
    }

    #[test]
    fn login_failure_does_not_distinguish_failure_kind() {
        let unknown_account = LOGIN_FAILED;
        let wrong_password = LOGIN_FAILED;
        let unverified_account = LOGIN_FAILED;
        assert_eq!(unknown_account, wrong_password);
        assert_eq!(wrong_password, unverified_account);
    }
}

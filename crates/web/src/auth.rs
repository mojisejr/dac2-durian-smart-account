use leptos::prelude::*;

/// Compile proof for the extraction boundary between Leptos server functions
/// and the `axum-login` request extension. Slice 4 supplies real credentials.
#[server]
pub async fn auth_stack_probe() -> Result<bool, ServerFnError> {
    let auth_session = leptos_axum::extract::<store::StackAuthSession>().await?;
    Ok(auth_session.user.is_some())
}

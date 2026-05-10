use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

// Modules
mod components;
mod pages;
mod roles;

// Top-Level pages
use crate::pages::home::Home;

// Reads the pathname of `document.baseURI` (set by `<base data-trunk-public-url />`
// in index.html) and trims any trailing slash so leptos_router strips the prefix
// to "/" rather than "" — empty does not match `path!("/")`.
fn document_base_path() -> String {
    let Some(window) = web_sys::window() else { return String::new() };
    let Some(document) = window.document() else { return String::new() };
    let Ok(Some(base_uri)) = document.base_uri() else { return String::new() };
    let Ok(url) = web_sys::Url::new(&base_uri) else { return String::new() };
    url.pathname().trim_end_matches('/').to_string()
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

        // sets the document title
        <Title text="Welcome to Leptos CSR" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router base=document_base_path()>
            <Routes fallback=|| view! { NotFound }>
                <Route path=path!("/") view=Home />
            </Routes>
        </Router>
    }
}

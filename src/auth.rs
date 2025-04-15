use reqwest::blocking::{Certificate, Identity, RequestBuilder};
use reqwest::blocking::ClientBuilder; // Added for potential client configuration
use std::fs; // Added for reading certificate files
use std::path::Path; // Added for path handling

pub trait Authenticator: Send + Sync {
    fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder;
}

pub struct TokenAuth {
    pub token: String,
}

impl Authenticator for TokenAuth {
    fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder {
        builder.bearer_auth(&self.token)
    }
}

pub struct OAuth {
    pub access_token: String,
}
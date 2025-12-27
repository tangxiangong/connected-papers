use reqwest::{Client, RequestBuilder};

pub(crate) struct APIKey {
    pub(crate) header: String,
    pub(crate) value: String,
}

pub(crate) fn build_request(
    client: &Client,
    method: Method,
    url: &str,
    api_key: Option<APIKey>,
) -> RequestBuilder {
    let mut req_builder = match method {
        Method::Get => client.get(url),
        Method::Post => client.post(url),
    };
    if let Some(APIKey { header, value }) = api_key {
        req_builder = req_builder.header(header, value);
    }
    req_builder
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Method {
    Get,
    Post,
}

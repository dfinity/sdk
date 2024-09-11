use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::named_canister::get_ui_canister_url;
use anyhow::Context;
use candid::Principal;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use fn_error_context::context;
use std::net::{Ipv4Addr, Ipv6Addr};
use url::Host::{Domain, Ipv4, Ipv6};
use url::Url;

#[context("Failed to construct frontend url for canister {} on network '{}'.", canister_id, network.name)]
pub fn construct_frontend_url(
    network: &NetworkDescriptor,
    canister_id: &Principal,
) -> DfxResult<(String, Option<String>)> {
    let mut url = Url::parse(&network.providers[0]).with_context(|| {
        format!(
            "Failed to parse url for network provider {}.",
            &network.providers[0]
        )
    })?;
    // For localhost defined by IP address we suggest `<canister_id>.localhost` as an alternate way of accessing the canister because it plays nicer with SPAs.
    // We still display `<IP>?canisterId=<canister_id>` because Safari does not support localhost subdomains
    let url2 = if url.host() == Some(Ipv4(Ipv4Addr::LOCALHOST))
        || url.host() == Some(Ipv6(Ipv6Addr::LOCALHOST))
    {
        let mut subdomain_url = url.clone();
        let localhost_with_subdomain = format!("{}.localhost", canister_id);
        subdomain_url
            .set_host(Some(&localhost_with_subdomain))
            .with_context(|| format!("Failed to set host to {}.", localhost_with_subdomain))?;
        Some(url2_string)
    } else {
        None
    };

    if let Some(Domain(domain)) = url.host() {
        let host = format!("{}.{}", canister_id, domain);
        url.set_host(Some(&host))
            .with_context(|| format!("Failed to set host to {}.", host))?;
    } else {
        let query = format!("canisterId={}", canister_id);
        url.set_query(Some(&query));
    };

    Ok((url_string, url2))
}

#[context("Failed to construct ui canister url for {} on network '{}'.", canister_id, env.get_network_descriptor().name)]
pub fn construct_ui_canister_url(
    env: &dyn Environment,
    canister_id: &Principal,
) -> DfxResult<Option<String>> {
    let mut url = get_ui_canister_url(env)?;
    if let Some(base_url) = url.as_mut() {
        let query_with_canister_id = if let Some(query) = base_url.query() {
            format!("{query}&id={canister_id}")
        } else {
            format!("id={canister_id}")
        };
        base_url.set_query(Some(&query_with_canister_id));
    };
    Ok(url_string)
}

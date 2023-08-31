use url::Host::Domain;
use url::ParseError;
use url::Url;

const MAINNET_CANDID_INTERFACE_PRINCIPAL: &str = "a4gq6-oaaaa-aaaab-qaa4q-cai";

pub fn format_frontend_url(provider: &Url, canister_id: &str) -> Url {
    let mut url = Url::clone(&provider);
    if let Some(Domain(domain)) = url.host() {
        if domain.ends_with("icp-api.io") || domain.ends_with("ic0.app") {
            let new_domain = domain.replace("icp-api.io", "icp0.io");
            let new_domain = new_domain.replace("ic0.app", "icp0.io");
            let host = format!("{}.{}", canister_id, new_domain);
            let _ = url.set_host(Some(&host));
        } else {
            let host = format!("{}.{}", canister_id, domain);
            let _ = url.set_host(Some(&host));
        }
    } else {
        let query = format!("canisterId={}", canister_id);
        url.set_query(Some(&query));
    }
    url
}

pub fn format_ui_canister_url_ic(canister_id: &str) -> Result<Url, ParseError> {
    let url_result = Url::parse(
        format!(
            "https://{}.raw.icp0.io/?id={}",
            MAINNET_CANDID_INTERFACE_PRINCIPAL, canister_id
        )
        .as_str(),
    );
    return url_result;
}

pub fn format_ui_canister_url_custom(
    canister_id: &str,
    provider: &Url,
    ui_canister_id: &str,
) -> Url {
    let mut url = Url::clone(&provider);

    if let Some(Domain(domain)) = url.host() {
        let host = format!("{}.{}", ui_canister_id, domain);
        let query = format!("id={}", canister_id);
        url.set_host(Some(&host)).unwrap();
        url.set_query(Some(&query));
    } else {
        let query = format!("canisterId={}&id={}", ui_canister_id, canister_id);
        url.set_query(Some(&query));
    }

    return url;
}

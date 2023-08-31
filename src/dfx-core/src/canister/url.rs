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
        } 
        else if domain.contains("localhost") {
            let port = url.port().unwrap_or(4943);
            let host = format!("localhost:{}", port);
            let query = format!("canisterId={}", canister_id);
            url.set_host(Some(&host)).unwrap();
            url.set_query(Some(&query));
        }
        else {
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

#[cfg(test)]
mod test {
    use url::Url;
    use crate::canister::url::format_frontend_url;

    #[test]
    fn print_local_frontend() {
        let provider1 = &Url::parse("http://127.0.0.1:4943").unwrap();
        let provider2 = &Url::parse("http://localhost:4943").unwrap();
        let provider3 = &Url::parse("http://127.0.0.1:8000").unwrap();
        assert_eq!(
            format_frontend_url(provider1, "ryjl3-tyaaa-aaaaa-aaaba-cai").as_str(),
            "http://127.0.0.1:4943/?canisterId=ryjl3-tyaaa-aaaaa-aaaba-cai"
        );
        assert_eq!(
            format_frontend_url(provider2, "ryjl3-tyaaa-aaaaa-aaaba-cai").as_str(),
            "http://localhost:4943/?canisterId=ryjl3-tyaaa-aaaaa-aaaba-cai"
        );
        assert_eq!(
            format_frontend_url(provider3, "ryjl3-tyaaa-aaaaa-aaaba-cai").as_str(),
            "http://127.0.0.1:8000/?canisterId=ryjl3-tyaaa-aaaaa-aaaba-cai"
        );
    }

    #[test]
    fn print_ic_frontend() {
        let provider1 = &Url::parse("https://ic0.app").unwrap();
        let provider2 = &Url::parse("https://icp-api.io").unwrap();
        let provider3 = &Url::parse("https://icp0.io").unwrap();
        assert_eq!(
            format_frontend_url(provider1, "ryjl3-tyaaa-aaaaa-aaaba-cai").as_str(),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
        assert_eq!(
            format_frontend_url(provider2, "ryjl3-tyaaa-aaaaa-aaaba-cai").as_str(),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
        assert_eq!(
            format_frontend_url(provider3, "ryjl3-tyaaa-aaaaa-aaaba-cai").as_str(),
            "https://ryjl3-tyaaa-aaaaa-aaaba-cai.icp0.io/"
        );
    }
}

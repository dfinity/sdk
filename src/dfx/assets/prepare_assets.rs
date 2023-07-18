use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use backoff::future::retry;
use backoff::ExponentialBackoffBuilder;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tar::{Archive, Builder, EntryType, Header};
use tokio::task::{spawn, spawn_blocking, JoinSet};

use crate::Source;

#[tokio::main]
pub(crate) async fn prepare(out_dir: &Path, source_set: HashMap<String, Source>) {
    std::fs::create_dir_all(out_dir).expect("error creating output directory");
    let out_dir_ = out_dir.to_owned();
    let copy_join = spawn_blocking(|| copy_canisters(out_dir_));
    let out_dir = out_dir.to_owned();
    make_binary_cache(out_dir, source_set).await;
    copy_join.await.unwrap();
}

fn copy_canisters(out_dir: PathBuf) {
    let distributed = Path::new("../distributed");
    for can in ["assetstorage", "wallet", "ui"] {
        let mut tar = Builder::new(GzEncoder::new(
            BufWriter::new(File::create(out_dir.join(format!("{can}_canister.tgz"))).unwrap()),
            Compression::new(6),
        ));
        for ext in [
            ".did",
            if can == "assetstorage" {
                ".wasm.gz"
            } else {
                ".wasm"
            },
        ] {
            let filename = format!("{can}{ext}");
            let input_file = File::open(distributed.join(&filename)).unwrap();
            let metadata = input_file.metadata().unwrap();
            let mut header = Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(metadata.len());
            tar.append_data(&mut header, &filename, input_file).unwrap();
        }
        tar.finish().unwrap();
    }
}

async fn download_canisters(
    client: Client,
    sources: Arc<HashMap<String, Source>>,
    out_dir: PathBuf,
) {
    // replace with joinset setup from download_binaries if another gets added
    let source = sources["ic-btc-canister"].clone();
    let btc_canister = download_and_check_sha(client, source).await;
    spawn_blocking(move || {
        let mut tar = Builder::new(GzEncoder::new(
            BufWriter::new(File::create(out_dir.join("btc_canister.tgz")).unwrap()),
            Compression::new(6),
        ));
        let mut header = Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(btc_canister.len() as u64);
        tar.append_data(
            &mut header,
            "ic-btc-canister.wasm.gz",
            btc_canister.reader(),
        )
        .unwrap();
        tar.finish().unwrap();
    })
    .await
    .unwrap();
}

async fn make_binary_cache(out_dir: PathBuf, sources: HashMap<String, Source>) {
    let sources = Arc::new(sources);
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .unwrap();
    let mo_base = spawn(download_mo_base(client.clone(), sources.clone()));
    let bins = spawn(download_binaries(client.clone(), sources.clone()));
    let bin_tars = spawn(download_bin_tarballs(client.clone(), sources.clone()));
    let canisters = spawn(download_canisters(
        client.clone(),
        sources.clone(),
        out_dir.clone(),
    ));
    let (mo_base, bins, bin_tars, _) =
        tokio::try_join!(mo_base, bins, bin_tars, canisters).unwrap();
    spawn_blocking(|| write_binary_cache(out_dir, mo_base, bins, bin_tars))
        .await
        .unwrap();
}

fn write_binary_cache(
    out_dir: PathBuf,
    mo_base: HashMap<PathBuf, Bytes>,
    bins: HashMap<PathBuf, Bytes>,
    mut bin_tars: HashMap<PathBuf, Bytes>,
) {
    let mut tar = Builder::new(GzEncoder::new(
        BufWriter::new(File::create(out_dir.join("binary_cache.tgz")).unwrap()),
        Compression::new(6),
    ));
    for (path, bin) in bins.into_iter().chain(
        ["icx-proxy", "ic-ref", "moc", "mo-doc", "mo-ide"]
            .map(|bin| (bin.into(), bin_tars.remove(Path::new(bin)).unwrap())),
    ) {
        let mut header = Header::new_gnu();
        header.set_size(bin.len() as u64);
        header.set_mode(0o500);
        tar.append_data(&mut header, path, bin.reader()).unwrap();
    }

    for (path, file) in bin_tars {
        let mut header = Header::new_gnu();
        header.set_size(file.len() as u64);
        header.set_mode(0o644);
        tar.append_data(&mut header, path, file.reader()).unwrap();
    }
    let mut base_hdr = Header::new_gnu();
    base_hdr.set_entry_type(EntryType::dir());
    base_hdr.set_mode(0o755);
    base_hdr.set_size(0);
    tar.append_data(&mut base_hdr, "base", io::empty()).unwrap();
    for (path, file) in mo_base {
        let mut header = Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(file.len() as u64);
        tar.append_data(&mut header, Path::new("base").join(path), file.reader())
            .unwrap();
    }
    tar.finish().unwrap();
}

async fn download_and_check_sha(client: Client, source: Source) -> Bytes {
    let retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();

    let response = retry(retry_policy, || async {
        match client.get(&source.url).send().await {
            Ok(response) => Ok(response),
            Err(err) => Err(backoff::Error::transient(err)),
        }
    })
    .await
    .unwrap();

    response.error_for_status_ref().unwrap();
    let content = response.bytes().await.unwrap();
    let sha = Sha256::digest(&content);
    assert_eq!(
        sha[..],
        source.sha256()[..],
        "sha256 hash for {} did not match",
        source.url
    );
    content
}

async fn download_binaries(
    client: Client,
    sources: Arc<HashMap<String, Source>>,
) -> HashMap<PathBuf, Bytes> {
    let mut joinset = JoinSet::new();
    for bin in [
        "ic-admin",
        "ic-btc-adapter",
        "ic-https-outcalls-adapter",
        "ic-nns-init",
        "replica",
        "canister_sandbox",
        "sandbox_launcher",
        "ic-starter",
        "sns",
    ] {
        let source = sources
            .get(bin)
            .unwrap_or_else(|| panic!("Cannot find source for {bin}"))
            .clone();
        let client_ = client.clone();
        joinset.spawn(async move { (bin, download_and_check_sha(client_, source).await) });
    }
    let mut map = HashMap::new();
    while let Some(res) = joinset.join_next().await {
        let (bin, content) = res.unwrap();
        let decompressed = spawn_blocking(|| {
            let mut buf = BytesMut::new();
            io::copy(
                &mut GzDecoder::new(content.reader()),
                &mut (&mut buf).writer(),
            )
            .unwrap();
            buf.freeze()
        })
        .await
        .unwrap();
        map.insert(bin.into(), decompressed);
    }
    map
}

async fn download_bin_tarballs(
    client: Client,
    sources: Arc<HashMap<String, Source>>,
) -> HashMap<PathBuf, Bytes> {
    let mut map = HashMap::new();
    let [motoko, icx_proxy, ic_ref] = ["motoko", "icx-proxy", "ic-ref"].map(|pkg| {
        let client = client.clone();
        let source = sources[pkg].clone();
        spawn(download_and_check_sha(client, source))
    });
    let (motoko, icx_proxy, ic_ref) = tokio::try_join!(motoko, icx_proxy, ic_ref).unwrap();
    for tar in [motoko, icx_proxy, ic_ref] {
        tar_xzf(&tar, |path, content| {
            map.insert(path, content);
        });
    }
    map
}

async fn download_mo_base(
    client: Client,
    sources: Arc<HashMap<String, Source>>,
) -> HashMap<PathBuf, Bytes> {
    let source = sources["motoko-base"].clone();
    let mo_base = download_and_check_sha(client, source).await;
    let mut map = HashMap::new();
    tar_xzf(&mo_base, |path, content| {
        let path = path.strip_prefix(".").unwrap_or(&path); // normalize ./x to x
        if let Ok(file) = path.strip_prefix("src") {
            map.insert(file.to_owned(), content);
        }
    });
    map
}

fn tar_xzf(gz: &[u8], mut each: impl FnMut(PathBuf, Bytes)) {
    let mut tar = Archive::new(GzDecoder::new(gz));
    for entry in tar.entries().unwrap() {
        let mut entry = entry.unwrap();
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().unwrap_or_else(|e| {
            panic!(
                "Malformed file path {}: {e}",
                String::from_utf8_lossy(&entry.path_bytes())
            )
        });
        let path = path.strip_prefix(".").unwrap_or(&path).to_owned();
        let mut content = BytesMut::with_capacity(entry.header().size().unwrap() as usize);
        io::copy(&mut entry, &mut (&mut content).writer()).unwrap();
        each(path, content.freeze());
    }
}

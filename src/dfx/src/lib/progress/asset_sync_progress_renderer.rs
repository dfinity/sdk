use crate::lib::environment::Environment;
use crate::lib::progress_bar::ProgressBar;
use ic_asset::{AssetSyncProgressRenderer, AssetSyncState};
use std::cell::OnceCell;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct EnvAssetSyncProgressRenderer<'a> {
    pub env: &'a dyn Environment,

    topline: ProgressBar,
    bytes: OnceCell<ProgressBar>,

    // getting properties of assets already in canister
    total_assets_to_retrieve_properties: AtomicUsize,
    assets_retrieved_properties: AtomicUsize,

    // staging assets
    total_assets: AtomicUsize,
    complete_assets: AtomicUsize,

    // uploading content
    total_bytes: AtomicUsize,
    uploaded_bytes: AtomicUsize,

    // committing batch
    total_batch_operations: AtomicUsize,
    committed_batch_operations: AtomicUsize,
}

impl<'a> EnvAssetSyncProgressRenderer<'a> {
    pub fn new(env: &'a dyn Environment) -> Self {
        let topline = env.new_spinner("Synchronizing assets".into());

        let total_assets_to_retrieve_properties = AtomicUsize::new(0);
        let assets_retrieved_properties = AtomicUsize::new(0);

        let total_assets = AtomicUsize::new(0);
        let complete_assets = AtomicUsize::new(0);

        let bytes = OnceCell::new(); // env.new_spinner("Uploading content...".into());
        let total_bytes = AtomicUsize::new(0);
        let uploaded_bytes = AtomicUsize::new(0);

        let total_batch_operations = AtomicUsize::new(0);
        let committed_batch_operations = AtomicUsize::new(0);

        Self {
            env,
            topline,
            total_assets_to_retrieve_properties,
            assets_retrieved_properties,
            total_assets,
            complete_assets,
            bytes,
            total_bytes,
            uploaded_bytes,
            total_batch_operations,
            committed_batch_operations,
        }
    }

    fn get_bytes_progress_bar(&self) -> &ProgressBar {
        self.bytes
            .get_or_init(|| self.env.new_spinner("Uploading content...".into()))
    }

    fn update_get_asset_properties(&self) {
        let total = self
            .total_assets_to_retrieve_properties
            .load(Ordering::SeqCst);
        let got = self.assets_retrieved_properties.load(Ordering::SeqCst);
        self.topline
            .set_message(format!("Read asset properties: {}/{}", got, total).into());
    }

    fn update_assets(&self) {
        let total = self.total_assets.load(Ordering::SeqCst);
        let complete = self.complete_assets.load(Ordering::SeqCst);
        self.topline
            .set_message(format!("Staged: {}/{} assets", complete, total).into());
    }

    fn update_bytes(&self) {
        let uploaded = self.uploaded_bytes.load(Ordering::SeqCst);
        self.get_bytes_progress_bar()
            .set_message(format!("Uploaded content: {} bytes", uploaded).into());
    }

    fn update_commit_batch(&self) {
        let total = self.total_batch_operations.load(Ordering::SeqCst);
        let committed = self.committed_batch_operations.load(Ordering::SeqCst);
        self.topline
            .set_message(format!("Committed batch: {}/{} operations", committed, total).into());
    }
}

impl<'a> AssetSyncProgressRenderer for EnvAssetSyncProgressRenderer<'a> {
    fn set_state(&self, state: AssetSyncState) {
        if matches!(state, AssetSyncState::CommitBatch) {
            if let Some(bar) = self.bytes.get() {
                bar.finish_and_clear();
            }
        }
        if matches!(state, AssetSyncState::Done) {
            self.topline.finish_and_clear();
            return;
        }

        let msg = match state {
            AssetSyncState::GatherAssetDescriptors => "Gathering asset descriptors",
            AssetSyncState::ListAssets => "Listing assets",
            AssetSyncState::GetAssetProperties => "Getting asset properties",
            AssetSyncState::CreateBatch => "Creating batch",
            AssetSyncState::StageContents => "Staging contents",
            AssetSyncState::AssembleBatch => "Assembling batch",
            AssetSyncState::CommitBatch => "Committing batch",
            AssetSyncState::Done => unreachable!(),
        };
        self.topline.set_message(msg.into());
    }

    fn set_asset_properties_to_retrieve(&self, total: usize) {
        self.total_assets_to_retrieve_properties
            .store(total, Ordering::SeqCst);
    }
    fn inc_asset_properties_retrieved(&self) {
        self.assets_retrieved_properties
            .fetch_add(1, Ordering::SeqCst);
        self.update_get_asset_properties();
    }

    fn set_total_assets(&self, total: usize) {
        self.total_assets.store(total, Ordering::SeqCst);
        self.update_assets();
    }

    fn increment_complete_assets(&self) {
        self.complete_assets.fetch_add(1, Ordering::SeqCst);
        self.update_assets();
    }

    fn add_total_bytes(&self, add: usize) {
        self.total_bytes.fetch_add(add, Ordering::SeqCst);
        self.update_bytes();
    }
    fn add_uploaded_bytes(&self, add: usize) {
        self.uploaded_bytes.fetch_add(add, Ordering::SeqCst);
        self.update_bytes();
    }

    fn set_total_batch_operations(&self, total: usize) {
        self.total_batch_operations
            .fetch_add(total, Ordering::SeqCst);
    }

    fn add_committed_batch_operations(&self, add: usize) {
        self.committed_batch_operations
            .fetch_add(add, Ordering::SeqCst);
        self.update_commit_batch();
    }
}

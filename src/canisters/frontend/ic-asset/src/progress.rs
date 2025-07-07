/// .
#[derive(Debug)]
pub enum AssetSyncState {
    /// Walk the source directories and build a list of assets to sync
    GatherAssetDescriptors,

    /// List all assets already in the canister
    ListAssets,

    /// Get properties of all assets already in the canister
    GetAssetProperties,

    /// Create the batch
    CreateBatch,

    /// Upload content encodings (chunks)
    StageContents,

    /// Build the list of operations to apply all changes
    AssembleBatch,

    /// Commit (execute) batch operations
    CommitBatch,

    /// All done
    Done,
}

/// Display progress of the synchronization process
pub trait AssetSyncProgressRenderer: Send + Sync {
    /// Set the current state of the synchronization process
    fn set_state(&self, state: AssetSyncState);

    /// Set the total number of assets to get properties for
    fn set_asset_properties_to_retrieve(&self, total: usize);

    /// Increment the number of assets for which properties have been retrieved
    fn inc_asset_properties_retrieved(&self);

    /// Set the total number of assets to sync
    fn set_total_assets(&self, total: usize);

    /// Increment the number of assets that have been synced
    fn increment_complete_assets(&self);

    /// Set the total number of bytes to upload
    fn add_total_bytes(&self, add: usize);

    /// Increment the number of bytes that have been uploaded
    fn add_uploaded_bytes(&self, add: usize);

    /// Set the total number of batch operations
    fn set_total_batch_operations(&self, total: usize);

    /// Increase the number of batch operations that have been committed
    fn add_committed_batch_operations(&self, add: usize);
}

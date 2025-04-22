use super::bindings::icp::host::host;

use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

//
// Host
//

/// Plugins host.
pub struct Host {
    wasi: WasiCtx,
    resources: ResourceTable,
}

impl Host {
    /// Constructor.
    pub fn new() -> Self {
        let wasi = WasiCtxBuilder::new().inherit_stdout().build();
        Self {
            wasi,
            resources: ResourceTable::new(),
        }
    }
}

// We need to implement WasiView for wasmtime_wasi::add_to_linker_sync
impl WasiView for Host {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl IoView for Host {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

// Our exposed Host functions
impl host::Host for Host {
    fn log(&mut self, message: String) {
        println!("log: {}", message);
    }
}

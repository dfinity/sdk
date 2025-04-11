// Note that we are using the full notation of the macro in order
// to add some options that allow the bindings to live in this independent file

#[allow(clippy::crate_in_macro_def)]
wit_bindgen::generate!({
    path: "../../assets/wit/acme-plugins.wit",
    default_bindings_module: "crate::bindings",
    pub_export_macro: true,
});

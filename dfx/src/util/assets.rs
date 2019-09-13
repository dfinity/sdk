include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

pub fn color_logo() -> String {
    include_str!("../../assets/dfinity.aart").to_string()
}

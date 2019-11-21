include!(concat!(env!("OUT_DIR"), "/load_assets.rs"));

pub fn dfinity_logo() -> String {
    if atty::is(atty::Stream::Stdout) {
        include_str!("../../assets/dfinity-color.aart").to_string()
    } else {
        include_str!("../../assets/dfinity-nocolor.aart").to_string()
    }
}

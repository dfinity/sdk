extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process_current_dir()
        .unwrap();    
}

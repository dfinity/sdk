extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .force_build(true)
        .generate_in_source_tree()        
        .process()
        .unwrap();
}

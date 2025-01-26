use apply_patch::apply_to;
use patch::Patch;

#[test]
fn lao_tzu() {
    let patch = include_str!("example.patch");
    let lao = include_str!("lao.txt");
    let tzu = include_str!("tzu.txt");
    let patch = Patch::from_single(patch).unwrap();
    let patched = apply_to(&patch, lao).unwrap();
    assert_eq!(patched, tzu);
}

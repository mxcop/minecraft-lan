use mc_lan_test::scan_lan;

fn main() {
    let servers = scan_lan().unwrap();

    dbg!(servers);
}
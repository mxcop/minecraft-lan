use minecraft_lan::scan_lan;

fn main() {
    let servers = scan_lan().unwrap();

    dbg!(servers);
}
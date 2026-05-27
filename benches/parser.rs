use sphinx_inv::SphinxInventoryReader;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("objects.inv");
    let _: Vec<_> = SphinxInventoryReader::from_path(path).iter().collect();
}

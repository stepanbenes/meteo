mod rtl;

fn main() {
    rtl::start().expect("Failed to start RTL-433 parser");
}

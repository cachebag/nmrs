#[tokio::main(flavor = "current_thread")]
async fn main() {
    nmrs_gui::run().ok();
}

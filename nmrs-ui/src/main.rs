#[tokio::main(flavor = "current_thread")]
async fn main() {
    nmrs_ui::run().ok();
}

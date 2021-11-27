feature_cfg! {
    for "futures-lock";

    pub use futures_test::test;
}

feature_cfg! {
    for "tokio-lock";

    pub use tokio::test;
}

feature_cfg! {
    for "async-std-lock";

    pub use async_std::test;
}

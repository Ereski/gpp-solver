family_cfg! {
    for "wasm";

    // TODO: add `run_in_browser`
    pub use wasm_bindgen_test::wasm_bindgen_test as test;
}

family_cfg! {
    for !"wasm";

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
}

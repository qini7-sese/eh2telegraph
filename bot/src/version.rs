pub(crate) static VERSION: &str = concat!(
    "\n",
    "Build Timestamp:\t",
    env!("VERGEN_BUILD_TIMESTAMP"),
    "\n",
    "Package Version:\t",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "rustc Version:  \t",
    env!("VERGEN_RUSTC_SEMVER"),
    "\n",
    "Cargo Target:   \t",
    env!("VERGEN_CARGO_TARGET_TRIPLE"),
    "\n",
    "Source code:    \thttps://github.com/qini7-sese/eh2telegraph"
);

use std::env::var;

fn main() {
    let is_msmpi = build_probe_mpi::probe().map_or(false, |lib| lib.version == "MS-MPI");

    if is_msmpi {
        println!("cargo:rustc-cfg=msmpi");

        if var("CARGO_FEATURE_USER_OPERATIONS").is_ok()
            && var("CARGO_CFG_TARGET_ARCH") == Ok("x86".to_string())
        {
            panic!(
                "Feature 'user-operations' is not supported for MS-MPI on 32-bit Windows. \
            See: https://github.com/rsmpi/rsmpi/issues/97"
            )
        }
    }
}

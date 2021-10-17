/// Generates the signer seeds for a [crate::state::CrateToken].
#[macro_export]
macro_rules! gen_crate_signer_seeds {
    ($ctoken:expr) => {
        &[
            b"CrateToken".as_ref(),
            $ctoken.mint.as_ref(),
            &[$ctoken.bump],
        ]
    };
}

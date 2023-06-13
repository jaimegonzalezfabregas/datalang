// #[macro_export]
// macro_rules! priv_printdev {
//     ( $($arg:expr),+) => {{
//         println!( $($arg),+)
//     }};
// }

extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn if_dev(ret: TokenStream) -> TokenStream {
    ret
}

#[proc_macro]
pub fn printdev(input: TokenStream) -> TokenStream {
    format!("println!({})", input.to_string()).parse().unwrap()
}

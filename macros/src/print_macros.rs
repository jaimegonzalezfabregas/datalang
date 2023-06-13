extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn if_dev(ret: TokenStream) -> TokenStream {
    if cfg!(debug_assertions) {
        ret
    } else {
        "".parse().unwrap()
    }
}

#[proc_macro]
pub fn printdev(input: TokenStream) -> TokenStream {
    format!(
        "
        if_dev!({{
            use backtrace::Backtrace;
            let depth = Backtrace::new_unresolved().frames().len();
            let mut margin = String::new();
            for _ in 0..depth{{
                margin += &\" \";
            }}
            print!(\"{{}}\",margin);

            println!({})
        }})",
        input.to_string()
    )
    .parse()
    .unwrap()
}

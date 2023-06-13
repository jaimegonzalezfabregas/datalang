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

            let mut depth = 0;
            
            let mut margin = String::new();

            backtrace::trace(|_| {{
                margin += &\" \";
                true
            }});
            
            print!(\"{{}}\",margin);

            println!({})
        }})",
        input.to_string()
    )
    .parse()
    .unwrap()
}

#[proc_macro]
pub fn printparse(input: TokenStream) -> TokenStream {
    //printdev(input)
    "".parse().unwrap()
}

#[proc_macro]
pub fn printprocess(input: TokenStream) -> TokenStream {
    //printdev(input)
    "".parse().unwrap()
}

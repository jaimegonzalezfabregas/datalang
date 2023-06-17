extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn printdev(input: TokenStream) -> TokenStream {
    if cfg!(debug_assertions) {
        format!(
            "
        {{
            use backtrace::Backtrace;

            let mut depth = 0;
            
            let mut margin = String::new();

            backtrace::trace(|_| {{
                margin += &\" \";
                true
            }});
            
            print!(\"{{}}\",margin);

            println!({})
        }}",
            input.to_string()
        )
        .parse()
        .unwrap()
    } else {
        "".parse().unwrap()
    }
}

#[proc_macro]
pub fn printparse(_input: TokenStream) -> TokenStream {
    //printdev(_input)
    "".parse().unwrap()
}

#[proc_macro]
pub fn printprocess(_input: TokenStream) -> TokenStream {
    //printdev(_input)
    "".parse().unwrap()
}

#[proc_macro]
pub fn printprocessop(_input: TokenStream) -> TokenStream {
    //printdev(_input)
    "".parse().unwrap()
}

#[proc_macro]
pub fn printinfo(_input: TokenStream) -> TokenStream {
    //printdev(_input)
    "".parse().unwrap()
}

#[proc_macro]
pub fn if_multithread(_input: TokenStream) -> TokenStream {
    _input
    //"".parse().unwrap()
}

#[proc_macro]
pub fn if_not_multithread(_input: TokenStream) -> TokenStream {
    //_input
    "".parse().unwrap()
}
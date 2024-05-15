use proc_macro::TokenStream;
use quote::quote;

const WIT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../wit");

#[proc_macro_attribute]
pub fn messaging_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let await_postfix = func.sig.asyncness.map(|_| quote!(.await));
    let preamble = preamble();

    quote!(
        #func
        pub mod __spin_messaging {
            pub mod preamble {
                #preamble
            }
        }
    ).into()
}

// impl self::preamble::Guest for preamble::Messaging {
//     fn handler(ms: ::spin_messaging_sdk::Message) -> ::std::result::Result<(), ::spin_messaging_sdk::Error> {
//         ::spin_messaging_sdk::executor::run(async move {
//             match super::#func_name(ms)#await_postfix {
//                 ::std::result::Result::Ok(()) => ::std::result::Result::Ok(()),
//                 ::std::result::Result::Err(e) => {
//                     eprintln!("{}", e);
//                     ::std::result::Result::Err(::spin_messaging_sdk::Error::Other(e.to_string()))
//                 },
//             }
//         })
//     }
// }

fn preamble() -> proc_macro2::TokenStream {
    let world = "messaging";
    quote! {
        #![allow(missing_docs)]
        ::spin_messaging_sdk::wit_bindgen::generate!({
            world: #world,
            path: #WIT_PATH,
            runtime_path: "::spin_messaging_sdk::wit_bindgen::rt",
            exports: {
                "wasi:messaging/messaging-guest": MessagingGuest,
            },
            with: {
                "wasi:messaging/messaging-types": ::spin_messaging_sdk,
            }
        });
        pub struct Messaging;
        pub struct MessagingGuest;
    }
}
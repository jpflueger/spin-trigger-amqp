use proc_macro::TokenStream;
use quote::quote;

const WIT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../spin-amqp.wit");

#[proc_macro_attribute]
pub fn amqp_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let await_postfix = func.sig.asyncness.map(|_| quote!(.await));
    let preamble = preamble();

    quote!(
        #func
        mod __spin_amqp {
            mod preamble {
                #preamble
            }
            impl self::preamble::Guest for preamble::Amqp {
                fn handler(messages: ::spin_amqp_sdk::wit_bindgen::rt::vec::Vec<::spin_amqp_sdk::Message>) -> ::std::result::Result<(), ::spin_amqp_sdk::Error> {
                    ::spin_amqp_sdk::executor::run(async move {
                        match super::#func_name(messages)#await_postfix {
                            ::std::result::Result::Ok(()) => ::std::result::Result::Ok(()),
                            ::std::result::Result::Err(e) => {
                                eprintln!("{}", e);
                                ::std::result::Result::Err(::spin_amqp_sdk::Error::Other(e.to_string()))
                            },
                        }
                    })
                }
            }
        }
    ).into()
}

fn preamble() -> proc_macro2::TokenStream {
    let world = "spin-amqp";
    quote! {
        #![allow(missing_docs)]
        ::spin_amqp_sdk::wit_bindgen::generate!({
            world: #world,
            path: #WIT_PATH,
            runtime_path: "::spin_amqp_sdk::wit_bindgen::rt",
            exports: {
                world: Amqp,
            },
            with: {
                "spin:amqp-trigger/spin-amqp-types": ::spin_amqp_sdk,
            }
        });
        pub struct Amqp;
    }
}

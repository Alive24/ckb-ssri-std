#![no_std]

extern crate alloc;
extern crate proc_macro;

mod molecule_schema;

use core::panic;

use ckb_hash::blake2b_256;
use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;
use syn::{parse::Parse, parse_macro_input, Expr, ExprLit, Ident, Lit, Token};

use alloc::vec;
use alloc::vec::Vec;

fn encode_u64_vector(val: impl AsRef<[u64]>) -> Vec<u8> {
    let val = val.as_ref();
    u32::to_le_bytes(val.len() as u32)
        .into_iter()
        .chain(val.iter().flat_map(|v| u64::to_le_bytes(*v)))
        .collect()
}

fn method_path(name: impl AsRef<[u8]>) -> u64 {
    u64::from_le_bytes(blake2b_256(name)[0..8].try_into().unwrap())
}

struct Methods {
    argv: Expr,
    invalid_method: Expr,
    invalid_args: Expr,
    method_keys: Vec<u64>,
    method_bodies: Vec<Expr>,
}

impl Parse for Methods {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let argv = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let invalid_method = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let invalid_args = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;

        let mut method_keys = vec![];
        let mut method_bodies = vec![];
        while !input.is_empty() {
            let name = match input.parse::<Expr>()? {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(v), ..
                }) => v.value(),
                _ => panic!("method name should be a string"),
            };
            input.parse::<Token![=>]>()?;
            let body = input.parse::<Expr>()?;
            input.parse::<Token![,]>()?;

            method_keys.push(method_path(name));
            method_bodies.push(body);
        }

        Ok(Methods {
            argv,
            invalid_method,
            invalid_args,
            method_keys,
            method_bodies,
        })
    }
}

#[proc_macro]
pub fn ssri_methods(input: TokenStream) -> TokenStream {
    let Methods {
        argv,
        invalid_method,
        invalid_args,
        method_keys,
        method_bodies,
    } = parse_macro_input!(input as Methods);

    let version_path = method_path("SSRI.version");
    let get_methods_path = method_path("SSRI.get_methods");
    let has_methods_path = method_path("SSRI.has_methods");

    let raw_methods = encode_u64_vector(
        [version_path, get_methods_path, has_methods_path]
            .iter()
            .chain(method_keys.iter())
            .copied()
            .collect::<Vec<_>>(),
    );
    let raw_methods_len = raw_methods.len();

    TokenStream::from(quote! {
        {
            use alloc::{borrow::Cow, vec::Vec};
            use ckb_std::high_level::decode_hex;
            const RAW_METHODS: [u8; #raw_methods_len] = [#(#raw_methods,)*];
            let res: Result<Cow<'static, [u8]>, Error> = match u64::from_le_bytes(
                decode_hex(&(#argv)[0])?.try_into().map_err(|_| #invalid_method)?,
            ) {
                #version_path => Ok(Cow::from(&[0][..])),
                #get_methods_path => {
                    let offset = usize::min((u64::from_le_bytes(
                        decode_hex(&(#argv)[1])?
                            .try_into()
                            .map_err(|_| #invalid_args)?
                    ) as usize * 8), #raw_methods_len - 4);

                    // If second argument is 0, take all remaining methods
                    let second_arg = u64::from_le_bytes(
                        decode_hex(&(#argv)[2])?
                            .try_into()
                            .map_err(|_| #invalid_args)?
                    );

                    let mut raw_result: Vec<u8>;
                    if second_arg == 0 {
                        // Take all remaining methods from offset
                        raw_result = RAW_METHODS[(offset + 4)..].to_vec();
                        let method_count = (RAW_METHODS.len() - (offset + 4)) / 8;
                        let mut result = (method_count as u32).to_le_bytes().to_vec();
                        result.extend_from_slice(&raw_result);
                        Ok(Cow::from(result))
                    } else {
                        let limit = usize::min((offset + (second_arg as usize * 8)), #raw_methods_len - 4);
                        raw_result = RAW_METHODS[(offset + 4)..(limit + 4)].to_vec();
                        let method_count = (limit - offset) / 8;
                        let mut result = (method_count as u32).to_le_bytes().to_vec();
                        result.extend_from_slice(&raw_result);
                        Ok(Cow::from(result))
                    }
                },
                #has_methods_path => {
                    let mut result = Vec::new();
                    let matches = decode_hex(&(#argv)[1])?[4..].chunks(8).map(|path| {
                        match RAW_METHODS[4..]
                            .chunks(8)
                            .find(|v| v == &path) {
                                Some(_) => 1,
                                None => 0,
                            }
                    }).collect::<Vec<_>>();
                    result.extend_from_slice(&(matches.len() as u32).to_le_bytes());
                    result.extend_from_slice(&matches);
                    Ok(Cow::from(result))
                },
                #(
                    #method_keys => #method_bodies,
                )*
                _ => Err(#invalid_method),
            };
            res
        }
    })
}

#[proc_macro_attribute]
pub fn molecule_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let schema = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as syn::ItemStruct);
    
    let struct_name = &input.ident;
    let schema_content = schema.value();
    
    let codec_map = molecule_schema::CodecMap::parse_schema(&schema_content)
        .expect("Failed to parse molecule schema");
        
    molecule_schema::generate_molecule_code(codec_map, struct_name)
}

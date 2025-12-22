use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Error, Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

use crate::list_chunk::find_vec_field;

#[derive(Default)]
struct MacroFlags {
    name_exception: bool,
}

struct MacroArgs {
    chunk_name: LitStr,
    flags: MacroFlags,
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let chunk_name: LitStr = input.parse()?;
        let mut flags = MacroFlags::default();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let flag: Ident = input.parse()?;

            let flag_name = flag.to_string();
            match flag_name.as_str() {
                "name_exception" => flags.name_exception = true,
                _ => {
                    return Err(Error::new_spanned(
                        &flag,
                        format!("Unknown flag '{flag_name}'. Valid flags are: name_exception"),
                    ));
                },
            };
        }

        Ok(MacroArgs { chunk_name, flags })
    }
}

pub fn named_list_chunk(attr: TokenStream, item: TokenStream) -> TokenStream {
    let chunk_struct = parse_macro_input!(item as DeriveInput);

    let (elems_field, elem_type) = match find_vec_field(&chunk_struct) {
        Ok((n, t)) => (n, t),
        Err(err) => return err.to_compile_error().into(),
    };

    let chunk_type = &chunk_struct.ident;

    let args = parse_macro_input!(attr as MacroArgs);
    let chunk_name = args.chunk_name;

    let name_impl = if args.flags.name_exception {
        quote! {}
    } else {
        quote! {
            impl crate::gamemaker::elements::GMNamedElement for #elem_type {
                fn name(&self) -> &String {
                    &self.name
                }

                fn name_mut(&mut self) -> &mut String {
                    &mut self.name
                }
            }
        }
    };

    let expanded = quote! {
        #[macros::list_chunk(#chunk_name)]
        #chunk_struct

        #name_impl

        impl crate::gamemaker::elements::GMNamedListChunk for #chunk_type {
            fn ref_by_name(&self, name: &str) -> Result<crate::gamemaker::reference::GMRef<#elem_type>> {
                for (i, element) in self.#elems_field.iter().enumerate() {
                    if element.name == name {
                        return Ok(i.into());
                    }
                }

                let error_message: String = format!(
                    "Could not find {} with name {:?}",
                    stringify!(#elem_type),
                    name,
                );

                return Err(error_message.into());
            }

            fn by_name(&self, name: &str) -> Result<&#elem_type> {
                self.ref_by_name(name).map(|gmref| &self.#elems_field[gmref.index as usize])
            }

            fn by_name_mut(&mut self, name: &str) -> Result<&mut #elem_type> {
                self.ref_by_name(name).map(|gmref| &mut self.#elems_field[gmref.index as usize])
            }
        }
    };

    TokenStream::from(expanded)
}

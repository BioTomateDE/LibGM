extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields,
    GenericArgument, Ident, PathArguments, Type, TypePath, PathSegment, Path
};


/// Checks if a type is Option<Option<T>>
fn is_option_of_option(ty: &Type) -> bool {
    if let Type::Path(TypePath { path: Path { segments, .. }, .. }) = ty {
        if let Some(PathSegment { ident, arguments }) = segments.first() {
            if ident == "Option" {
                if let PathArguments::AngleBracketed(args) = arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        if let Type::Path(TypePath { path: Path { segments, .. }, .. }) = inner_ty {
                            return segments.first().map_or(false, |seg| seg.ident == "Option");
                        }
                    }
                }
            }
        }
    }
    false
}

#[proc_macro_attribute]
pub fn deserialize_option_option(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let fields = if let Data::Struct(data_struct) = &mut input.data {
        match &mut data_struct.fields {
            Fields::Named(fields_named) => &mut fields_named.named,
            _ => return TokenStream::from(quote! { #input }),
        }
    } else {
        return TokenStream::from(quote! { #input });
    };

    // Generate unique snake_case helper name
    let helper_name = {
        let ident_str = name.to_string();
        let snake_case = ident_str
            .chars()
            .flat_map(|c| {
                if c.is_uppercase() {
                    vec!['_', c.to_ascii_lowercase()]
                } else {
                    vec![c]
                }
            })
            .collect::<String>()
            .trim_start_matches('_')
            .to_string();

        Ident::new(
            &format!("__deserialize_opt_opt_{}", snake_case),
            name.span()
        )
    };

    // Generate helper function
    let helper_fn = quote! {
        #[doc(hidden)]
        #[inline]
        fn #helper_name<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
        where
            D: serde::Deserializer<'de>,
            T: serde::Deserialize<'de>,
        {
            ::core::option::Option::deserialize(deserializer).map(::core::option::Option::Some)
        }
    };

    // Modify fields
    for field in fields.iter_mut() {
        if is_option_of_option(&field.ty) {
            // Create string literal for helper name
            let helper_str = &helper_name.to_string();

            // Create combined serde attribute
            let attr: Attribute = syn::parse_quote! {
                #[serde(deserialize_with = #helper_str, default)]
            };

            field.attrs.push(attr);
        }
    }

    let output = quote! {
        #helper_fn
        #input
    };

    output.into()
}


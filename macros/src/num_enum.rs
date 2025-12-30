use proc_macro::TokenStream;

pub fn num_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let repr_type = match syn::parse::<syn::Type>(attr) {
        Ok(ty) => ty,
        Err(err) => return err.to_compile_error().into(),
    };

    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let expanded = quote::quote! {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq,
            ::num_enum::TryFromPrimitive, ::num_enum::IntoPrimitive,
        )]
        #[repr(#repr_type)]
        #input
    };

    TokenStream::from(expanded)
}

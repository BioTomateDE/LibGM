use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn num_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    if attr.is_empty() {
        return syn::Error::new_spanned(
            proc_macro2::TokenStream::new(),
            "num_enum requires a representation type, e.g. #[num_enum(i32)]",
        )
        .to_compile_error()
        .into();
    }

    let repr_type =
        syn::parse::<syn::Type>(attr).expect("Expected a representation type like i32, u8, etc.");

    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let expanded = quote::quote! {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq,
            num_enum::TryFromPrimitive, num_enum::IntoPrimitive,
        )]
        #[repr(#repr_type)]
        #input
    };

    TokenStream::from(expanded)
}

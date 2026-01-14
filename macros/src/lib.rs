//! Visit <https://docs.rs/libgm>.

mod list_chunk;
mod named_list_chunk;
mod num_enum;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn num_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    num_enum::num_enum(attr, item)
}

#[proc_macro_attribute]
pub fn named_list_chunk(attr: TokenStream, item: TokenStream) -> TokenStream {
    named_list_chunk::named_list_chunk(attr, item)
}

#[proc_macro_attribute]
pub fn list_chunk(attr: TokenStream, item: TokenStream) -> TokenStream {
    list_chunk::list_chunk(attr, item)
}

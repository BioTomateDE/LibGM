use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

pub fn list_chunk(attr: TokenStream, item: TokenStream) -> TokenStream {
    let chunk_struct = parse_macro_input!(item as DeriveInput);
    let (elem_name, elem_type) = find_vec_field(&chunk_struct);
    let chunk_type = &chunk_struct.ident;

    let chunk_name = parse_macro_input!(attr as syn::LitStr);

    let expanded = quote::quote! {
        #[derive(Debug, Clone, Default, PartialEq)]
        #chunk_struct

        impl std::ops::Deref for #chunk_type {
            type Target = Vec<#elem_type>;
            fn deref(&self) -> &Self::Target {
                &self.#elem_name
            }
        }

        impl std::ops::DerefMut for #chunk_type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.#elem_name
            }
        }

        impl crate::gamemaker::elements::GMChunkElement for #chunk_type {
            const NAME: crate::gamemaker::chunk::ChunkName = crate::gamemaker::chunk::ChunkName::new(#chunk_name);
            fn exists(&self) -> bool {
                self.exists
            }
        }


        impl #chunk_type {
            fn index_by_name(&self, name: &str) -> crate::error::Result<usize> {
                for (i, element) in self.#elem_name.iter().enumerate() {
                    if element.name == name {
                        return Ok(i);
                    }
                }

                let error_message: String = format!(
                    "Could not find {} with name {:?}",
                    stringify!(#elem_type), name,
                );

                return Err(crate::error::Error::new(error_message));
            }

            pub fn ref_by_name(&self, name: &str) -> crate::error::Result<crate::gamemaker::reference::GMRef<#elem_type>> {
                self.index_by_name(name).map(|i| i.into())
            }

            pub fn by_name(&self, name: &str) -> crate::error::Result<&#elem_type> {
                self.index_by_name(name).map(|index| &self.#elem_name[index])
            }

            pub fn by_name_mut(&mut self, name: &str) -> crate::error::Result<&mut #elem_type> {
                self.index_by_name(name).map(|index| &mut self.#elem_name[index])
            }
        }
    };

    TokenStream::from(expanded)
}

fn find_vec_field(chunk_struct: &DeriveInput) -> (&Ident, &syn::Type) {
    let fields = extract_fields(chunk_struct);
    let vec_fields = find_vec_fields(fields);

    if vec_fields.is_empty() {
        panic!("Could not find any struct field with type Vec");
    }

    if vec_fields.len() > 1 {
        panic!("Multiple fields with type Vec available");
    }

    vec_fields[0]
}

fn extract_fields(input: &DeriveInput) -> Vec<&syn::Field> {
    let Data::Struct(struct_data) = &input.data else {
        panic!("Expected struct (not enum or union)");
    };

    let Fields::Named(fields) = &struct_data.fields else {
        panic!("Expected named struct fields (not unnamed or unit)");
    };

    fields.named.iter().collect()
}

fn find_vec_fields(fields: Vec<&syn::Field>) -> Vec<(&Ident, &syn::Type)> {
    let mut vec_fields = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().expect("Field identifer not set");
        if is_vec_type(&field.ty) {
            let inner_type = extract_vec_inner_type(&field.ty);
            vec_fields.push((ident, inner_type));
        }
    }

    vec_fields
}

/// Check last segment is "Vec"
fn is_vec_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident == "Vec")
            .unwrap_or(false)
    } else {
        false
    }
}

fn extract_vec_inner_type(ty: &syn::Type) -> &syn::Type {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        inner_ty
    } else {
        panic!("Cannot extract inner type of Vec");
    }
}

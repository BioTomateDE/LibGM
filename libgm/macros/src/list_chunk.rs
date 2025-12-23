use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, Fields, Ident, LitStr, parse_macro_input};

pub fn list_chunk(attr: TokenStream, item: TokenStream) -> TokenStream {
    let chunk_struct = syn::parse_macro_input!(item as DeriveInput);

    let (elems_field, elem_type) = match find_vec_field(&chunk_struct) {
        Ok((n, t)) => (n, t),
        Err(err) => return err.to_compile_error().into(),
    };

    let chunk_name = parse_macro_input!(attr as LitStr);
    let chunk_type = &chunk_struct.ident;

    quote! {
        #[derive(Debug, Clone, Default, PartialEq)]
        #chunk_struct

        impl IntoIterator for #chunk_type {
            type Item = #elem_type;
            type IntoIter = std::vec::IntoIter<#elem_type>;

            fn into_iter(self) -> Self::IntoIter {
                self.#elems_field.into_iter()
            }
        }

        impl<'a> IntoIterator for &'a #chunk_type {
            type Item = &'a #elem_type;
            type IntoIter = core::slice::Iter<'a, #elem_type>;

            fn into_iter(self) -> Self::IntoIter {
                self.#elems_field.iter()
            }
        }

        impl<'a> IntoIterator for &'a mut #chunk_type {
            type Item = &'a mut #elem_type;
            type IntoIter = core::slice::IterMut<'a, #elem_type>;

            fn into_iter(self) -> Self::IntoIter {
                self.#elems_field.iter_mut()
            }
        }

        impl std::ops::Index<usize> for #chunk_type {
            type Output = #elem_type;
            fn index(&self, index: usize) -> &Self::Output {
                &self.#elems_field[index]
            }
        }

        impl std::ops::IndexMut<usize> for #chunk_type {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                &mut self.#elems_field[index]
            }
        }

        impl crate::gamemaker::elements::GMChunk for #chunk_type {
            const NAME: crate::gamemaker::chunk::ChunkName = crate::gamemaker::chunk::ChunkName::new(#chunk_name);

            fn exists(&self) -> bool {
                self.exists
            }
        }

        impl crate::gamemaker::elements::GMListChunk for #chunk_type {
            type Element = #elem_type;

            fn elements(&self) -> &Vec<#elem_type> {
                &self.#elems_field
            }

            fn elements_mut(&mut self) -> &mut Vec<#elem_type> {
                &mut self.#elems_field
            }

            fn iter(&self) -> core::slice::Iter<'_, Self::Element> {
                self.#elems_field.iter()
            }

            fn iter_mut(&mut self) -> core::slice::IterMut<'_, Self::Element> {
                self.#elems_field.iter_mut()
            }

            fn into_iter(self) -> std::vec::IntoIter<Self::Element> {
                self.#elems_field.into_iter()
            }
        }
    }.into()
}

pub fn find_vec_field(chunk_struct: &DeriveInput) -> syn::Result<(Ident, syn::Type)> {
    let syn::Data::Struct(struct_data) = &chunk_struct.data else {
        return Err(Error::new_spanned(
            chunk_struct,
            "Expected struct (not enum or union)",
        ));
    };
    let Fields::Named(fields) = &struct_data.fields else {
        return Err(Error::new_spanned(
            &struct_data.fields,
            "Expected named struct fields (not unnamed or unit)",
        ));
    };
    let fields_list = fields.named.iter().collect();
    let found_fields = find_vec_fields(fields_list)?;

    if found_fields.is_empty() {
        return Err(Error::new_spanned(
            fields,
            "Could not find any struct field with type Vec",
        ));
    }
    if found_fields.len() > 1 {
        return Err(Error::new_spanned(
            fields,
            "Multiple fields with type Vec available",
        ));
    }

    Ok(found_fields[0].clone())
}

fn find_vec_fields(fields: Vec<&syn::Field>) -> syn::Result<Vec<(Ident, syn::Type)>> {
    let mut vec_fields = Vec::new();

    for field in fields {
        // This should never fail since the fields were already verified to be named.
        let ident = field.ident.as_ref().expect("Field identifer not set");

        if !is_vec_type(&field.ty) {
            continue;
        }

        let inner_type = extract_vec_inner_type(&field.ty);
        let Some(inner_type) = inner_type else {
            return Err(Error::new_spanned(field, "Cannot extract inner Vec type"));
        };

        vec_fields.push((ident.clone(), inner_type.clone()));
    }

    Ok(vec_fields)
}

/// Check last segment is "Vec"
fn is_vec_type(ty: &syn::Type) -> bool {
    let syn::Type::Path(type_path) = ty else {
        return false;
    };
    type_path
        .path
        .segments
        .last()
        .map(|seg| seg.ident == "Vec")
        .unwrap_or(false)
}

fn extract_vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        Some(inner_ty)
    } else {
        None
    }
}

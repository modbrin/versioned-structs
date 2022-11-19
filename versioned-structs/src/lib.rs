use std::cmp;

use bae::FromAttributes;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, punctuated::Punctuated, ItemStruct};

#[derive(Debug, Eq, PartialEq, FromAttributes)]
struct VersionedField {
    /// Starting version for field.
    from: syn::LitInt,
    /// Last version for field.
    to: Option<syn::LitInt>,
}

type VerType = usize;
static FIELD_ATTR_NAME: &str = "versioned_field";

/// Parse field attribute arguments, in key=value format.
/// Expects `from` value and optional `to` value.
fn extract_version_range(attrs: &[syn::Attribute]) -> Option<(VerType, Option<VerType>)> {
    let vf = VersionedField::from_attributes(attrs).ok()?;
    let from = vf.from.base10_parse::<VerType>().ok()?;
    let to = if let Some(to) = vf.to {
        Some(to.base10_parse::<VerType>().ok()?)
    } else {
        None
    };
    Some((from, to))
}

/// Check if attribute is a `versioned_field`.
fn is_version_attribute(attr: &syn::Attribute) -> bool {
    attr.path
        .segments
        .first()
        .map(|v| v.ident.to_string().as_str() == FIELD_ATTR_NAME)
        .unwrap_or(false)
}

fn attrib_versioned_impl(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let mut struct_stream = TokenStream::default();
    let mut max_version = 1;

    // check if custom field attribute is only set at most once for each field
    item.fields.iter().for_each(|field| {
        assert!(
            field
                .attrs
                .iter()
                .filter(|v| is_version_attribute(v))
                .count()
                <= 1,
            "`versioned_field` attribute encountered muliple times for one field"
        );
    });

    // select max version from attributes to determine number of generated structs
    macro_rules! select_max_version {
        ($struct_type:ident) => {
            $struct_type.$struct_type.iter().for_each(|field| {
                if let Some((from, to)) = extract_version_range(&field.attrs) {
                    max_version = cmp::max(max_version, from);
                    if let Some(to) = to {
                        max_version = cmp::max(max_version, to);
                    }
                }
            })
        };
    }

    match &item.fields {
        syn::Fields::Named(ref named) => {
            select_max_version!(named);
        }
        syn::Fields::Unnamed(ref unnamed) => {
            select_max_version!(unnamed)
        }
        syn::Fields::Unit => {}
    }

    // generate modified structs
    for current_ver in 1..=max_version {
        let mut current_item = item.clone();

        // add version suffix if applicable
        if current_ver != max_version {
            let mut new_ident = current_item.ident.to_string();
            new_ident.push_str(format!("V{}", current_ver).as_str());
            current_item.ident = syn::Ident::new(&new_ident, current_item.ident.span());
        }

        // remove fields which don't fit in version range
        macro_rules! filter_versioned_fields {
            ($struct_type:ident) => {
                $struct_type.$struct_type = Punctuated::from_iter(
                    $struct_type.$struct_type.clone().into_iter().filter(|f| {
                        extract_version_range(&f.attrs)
                            .map(|(from, to)| {
                                assert!(
                                    from <= to.unwrap_or(from),
                                    "`from` value must be greater than `to`"
                                );
                                current_ver >= from && to.map(|v| current_ver <= v).unwrap_or(true)
                            })
                            .unwrap_or(true)
                    }),
                )
            };
        }

        match &mut current_item.fields {
            syn::Fields::Named(ref mut named) => {
                filter_versioned_fields!(named);
            }
            syn::Fields::Unnamed(unnamed) => {
                filter_versioned_fields!(unnamed);
            }
            syn::Fields::Unit => {}
        }

        // remove custom attributes, they're not needed in generated structs
        current_item
            .fields
            .iter_mut()
            .for_each(|field| field.attrs.retain(|attr| !is_version_attribute(attr)));

        struct_stream.extend::<TokenStream>(current_item.into_token_stream().into());
    }

    struct_stream
}

#[proc_macro_attribute]
pub fn versioned(args: TokenStream, input: TokenStream) -> TokenStream {
    attrib_versioned_impl(args, input)
}

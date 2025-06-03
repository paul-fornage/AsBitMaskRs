extern crate proc_macro;
use proc_macro::TokenStream;

use syn::{parse_macro_input, Data, DeriveInput, Fields};
use quote::quote;

/// Automatically implements the AsBitMask trait for structs with boolean fields.
///
/// This macro will generate implementations for:
/// - `as_bytes`: Converts the boolean fields to a byte array representation
/// - `from_bytes`: Constructs the struct from a byte array representation
///
/// The number of bytes in the array is calculated based on the number of fields.
#[proc_macro_derive(AsBitMask)]
pub fn derive_as_bit_mask(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let struct_name = &input.ident;

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => fields_named,
                _ => panic!("AsBitMask derive only supports structs with named fields"),
            }
        },
        _ => panic!("AsBitMask derive only supports structs"),
    };

    // Collect field names
    let mut field_names = Vec::new();
    for field in &fields.named {
        if let Some(ident) = &field.ident {
            field_names.push(ident);
        }
    }

    // Calculate number of bytes needed
    let num_fields = field_names.len();
    let num_bytes = (num_fields + 7) / 8; // Ceiling division by 8

    // Generate the expressions for as_bytes method
    let mut as_bytes_expressions = Vec::new();
    for byte_index in 0..num_bytes {
        let mut byte_expr = Vec::new();
        for bit_pos in 0usize..8 {
            let field_index = byte_index * 8 + bit_pos;
            if field_index < num_fields {
                let field = &field_names[field_index];
                byte_expr.push(quote! {
                    ((self.#field as u8) << #bit_pos)
                });
            }
        }

        if !byte_expr.is_empty() {
            as_bytes_expressions.push(quote! {
                #(#byte_expr)|*
            });
        } else {
            as_bytes_expressions.push(quote! { 0 });
        }
    }

    // Generate the field initializers for from_bytes method
    let mut from_bytes_initializers = Vec::new();
    for (field_index, field) in field_names.iter().enumerate() {
        let byte_index = field_index / 8;
        let bit_pos: usize = field_index % 8;

        from_bytes_initializers.push(quote! {
            #field: (bytes[#byte_index] & (1 << #bit_pos as usize)) != 0
        });
    }

    // Generate the implementation
    let expanded = quote! {
        impl AsBitMask<#num_bytes> for #struct_name {
            fn as_bytes(&self) -> [u8; #num_bytes] {
                [#(#as_bytes_expressions),*]
            }

            fn from_bytes(bytes: &[u8; #num_bytes]) -> Self {
                #struct_name {
                    #(#from_bytes_initializers,)*
                }
            }
        }
    };

    // Return the generated implementation as a token stream
    expanded.into()
}



/// Automatically implements the AsBitMask trait for structs with boolean fields.
///
/// This macro will generate implementations for:
/// - `as_bytes`: Converts the boolean fields to a byte array representation
/// - `from_bytes`: Constructs the struct from a byte array representation
///
/// The number of bytes in the array is calculated based on the number of fields.
///
/// Example:
/// ```no_run
/// use crate::as_bit_mask_derive::AsBitMaskExplicit;
///
/// #[derive(AsBitMaskExplicit)]
/// pub struct MyStruct{
///     #[index(5)]
///     a: bool,
///     #[index(2)]
///     b: bool,
///     #[index(4)]
///     c: bool,
///     #[index(0)]
///     d: bool,
///     #[index(8)]
///     e: bool,
/// }
///
///
/// ```
#[proc_macro_derive(AsBitMaskExplicit, attributes(index))]
pub fn derive_as_bit_mask_explicit(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let struct_name = &input.ident;

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => fields_named,
                _ => panic!("AsBitMaskExplicit derive only supports structs with named fields"),
            }
        },
        _ => panic!("AsBitMaskExplicit derive only supports structs"),
    };

    // Collect field names and their explicit indices
    let mut field_data = Vec::new();
    for field in &fields.named {
        if let Some(ident) = &field.ident {
            // Look for the #[index(n)] attribute
            let mut index = None;
            for attr in &field.attrs {
                if attr.path().is_ident("index") {
                    // Parse the index value from the attribute
                    let index_value = attr.parse_args::<syn::LitInt>().expect("Index must be an integer");
                    index = Some(index_value.base10_parse::<usize>().expect("Failed to parse index as usize"));
                }
            }

            let idx = index.expect(&format!("Field '{}' is missing #[index(n)] attribute", ident));
            field_data.push((ident, idx));
        }
    }

    // Find the maximum bit index to determine the required number of bytes
    let max_index = field_data.iter()
        .map(|(_, idx)| *idx)
        .max()
        .unwrap_or(0);

    let num_bytes = (max_index + 8) / 8; // Ceiling division by 8

    // Generate the expressions for as_bytes method
    let mut as_bytes_expressions = Vec::new();
    for byte_index in 0..num_bytes {
        let byte_start = byte_index * 8;
        let byte_end = byte_start + 7;

        // Collect fields that belong to this byte
        let byte_fields: Vec<_> = field_data.iter()
            .filter(|(_, idx)| *idx >= byte_start && *idx <= byte_end)
            .collect();

        if byte_fields.is_empty() {
            as_bytes_expressions.push(quote! { 0 });
        } else {
            let field_expressions = byte_fields.iter().map(|(field, idx)| {
                let bit_pos = idx % 8;
                quote! {
                    ((self.#field as u8) << #bit_pos)
                }
            });

            as_bytes_expressions.push(quote! {
                #(#field_expressions)|*
            });
        }
    }

    // Generate the field initializers for from_bytes method
    let from_bytes_initializers = field_data.iter().map(|(field, idx)| {
        let byte_index = idx / 8;
        let bit_pos = idx % 8;

        quote! {
            #field: (bytes[#byte_index] & (1 << #bit_pos)) != 0
        }
    });

    // Generate the implementation
    let expanded = quote! {
        impl AsBitMask<#num_bytes> for #struct_name {
            fn as_bytes(&self) -> [u8; #num_bytes] {
                [#(#as_bytes_expressions),*]
            }

            fn from_bytes(bytes: &[u8; #num_bytes]) -> Self {
                #struct_name {
                    #(#from_bytes_initializers,)*
                }
            }
        }
    };

    // Return the generated implementation as a token stream
    expanded.into()
}

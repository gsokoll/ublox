use crate::types::packfield::PackField;
use crate::types::{PackDesc, PayloadLen};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Maximum tuple size supported by proptest's Strategy impl
const MAX_TUPLE_SIZE: usize = 12;

/// Generates proptest fuzz strategy code for a packet.
/// This creates `fuzz_payload_strategy()` and `fuzz_frame_strategy()` methods
/// that can be used to generate valid UBX frames for round-trip testing.
pub fn generate_fuzz_code_for_packet(pack_descr: &PackDesc) -> TokenStream {
    let pack_name = format_ident!("{}", pack_descr.name);
    let class = pack_descr.header.class;
    let id = pack_descr.header.id;

    let payload_len = match pack_descr.header.payload_len {
        PayloadLen::Fixed(len) => len as usize,
        PayloadLen::Max(len) => len as usize,
    };

    // Generate field strategies and serializers
    let mut field_strategies: Vec<TokenStream> = Vec::new();
    let mut field_names: Vec<syn::Ident> = Vec::new();
    let mut field_serializers: Vec<TokenStream> = Vec::new();

    for f in &pack_descr.fields {
        let field_name = format_ident!("f_{}", f.name);
        field_names.push(field_name.clone());

        // Generate strategy based on raw type and optional map_type
        let strategy = generate_strategy_for_field(f);
        field_strategies.push(strategy);

        // Generate serializer based on raw type
        let serializer = generate_serializer_for_type(&f.ty, &field_name);
        field_serializers.push(serializer);
    }

    // Handle empty structs
    if field_strategies.is_empty() {
        return quote! {
            #[cfg(any(test, feature = "fuzz"))]
            impl #pack_name {
                /// Generate a proptest strategy that produces valid payload bytes
                pub fn fuzz_payload_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                    proptest::strategy::Just(Vec::new())
                }

                /// Generate a proptest strategy that produces valid UBX frames
                pub fn fuzz_frame_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                    use proptest::strategy::Strategy;
                    Self::fuzz_payload_strategy().prop_map(|payload| {
                        crate::ubx_packets::fuzz_helpers::build_ubx_frame(#class, #id, &payload)
                    })
                }
            }
        };
    }

    // For packets with more than MAX_TUPLE_SIZE fields, we need to split into chunks
    // proptest only implements Strategy for tuples up to 12 elements
    if field_strategies.len() > MAX_TUPLE_SIZE {
        return generate_chunked_fuzz_code(
            &pack_name,
            class,
            id,
            payload_len,
            &field_strategies,
            &field_names,
            &field_serializers,
        );
    }

    // Generate chaos strategies (any value, for testing error handling)
    let mut chaos_strategies: Vec<TokenStream> = Vec::new();
    for f in &pack_descr.fields {
        let strategy = generate_strategy_for_type(&f.ty);
        chaos_strategies.push(strategy);
    }

    // Handle chunking for chaos strategies if needed
    let chaos_body = if chaos_strategies.len() > MAX_TUPLE_SIZE {
        let strategy_chunks: Vec<_> = chaos_strategies.chunks(MAX_TUPLE_SIZE).collect();
        let name_chunks: Vec<_> = field_names.chunks(MAX_TUPLE_SIZE).collect();
        
        let mut chunk_strategies: Vec<TokenStream> = Vec::new();
        let mut chunk_names: Vec<syn::Ident> = Vec::new();
        let mut flatten_code: Vec<TokenStream> = Vec::new();

        for (i, (strats, names)) in strategy_chunks.iter().zip(name_chunks.iter()).enumerate() {
            let chunk_name = format_ident!("chunk_{}", i);
            chunk_names.push(chunk_name.clone());
            let strat_list: Vec<_> = strats.iter().collect();
            chunk_strategies.push(quote! { (#(#strat_list),*) });
            let name_list: Vec<_> = names.iter().collect();
            flatten_code.push(quote! { let (#(#name_list),*) = #chunk_name; });
        }

        quote! {
            (
                #(#chunk_strategies),*
            ).prop_map(|(#(#chunk_names),*)| {
                #(#flatten_code)*
                let mut payload = Vec::with_capacity(#payload_len);
                #(#field_serializers)*
                payload
            })
        }
    } else {
        quote! {
            (
                #(#chaos_strategies),*
            ).prop_map(|(#(#field_names),*)| {
                let mut payload = Vec::with_capacity(#payload_len);
                #(#field_serializers)*
                payload
            })
        }
    };

    quote! {
        #[cfg(any(test, feature = "fuzz"))]
        impl #pack_name {
            /// Generate a proptest strategy that produces semantically valid payload bytes.
            /// Enum-mapped fields will only contain valid enum values.
            /// Use this for testing correct parsing of valid data.
            pub fn fuzz_payload_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::prelude::*;
                (
                    #(#field_strategies),*
                ).prop_map(|(#(#field_names),*)| {
                    let mut payload = Vec::with_capacity(#payload_len);
                    #(#field_serializers)*
                    payload
                })
            }

            /// Generate a proptest strategy that produces valid UBX frames with semantic validity.
            /// Use this for testing correct parsing of valid data.
            pub fn fuzz_frame_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::strategy::Strategy;
                Self::fuzz_payload_strategy().prop_map(|payload| {
                    crate::ubx_packets::fuzz_helpers::build_ubx_frame(#class, #id, &payload)
                })
            }

            /// Generate a proptest strategy that produces payload bytes with arbitrary values.
            /// Fields use full type range (e.g., 0-255 for u8) regardless of semantic validity.
            /// Use this for testing graceful error handling of invalid/unexpected data.
            pub fn fuzz_payload_chaos_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::prelude::*;
                #chaos_body
            }

            /// Generate a proptest strategy that produces UBX frames with arbitrary field values.
            /// Use this for testing graceful error handling of invalid/unexpected data.
            pub fn fuzz_frame_chaos_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::strategy::Strategy;
                Self::fuzz_payload_chaos_strategy().prop_map(|payload| {
                    crate::ubx_packets::fuzz_helpers::build_ubx_frame(#class, #id, &payload)
                })
            }
        }
    }
}

/// Generate fuzz code for packets with more than 12 fields by chunking strategies
fn generate_chunked_fuzz_code(
    pack_name: &syn::Ident,
    class: u8,
    id: u8,
    payload_len: usize,
    field_strategies: &[TokenStream],
    field_names: &[syn::Ident],
    field_serializers: &[TokenStream],
) -> TokenStream {
    // Split into chunks of MAX_TUPLE_SIZE
    let strategy_chunks: Vec<_> = field_strategies.chunks(MAX_TUPLE_SIZE).collect();
    let name_chunks: Vec<_> = field_names.chunks(MAX_TUPLE_SIZE).collect();

    // Generate nested tuple strategies for valid/semantic fuzzing
    let mut chunk_strategies: Vec<TokenStream> = Vec::new();
    let mut chunk_names: Vec<syn::Ident> = Vec::new();
    let mut flatten_code: Vec<TokenStream> = Vec::new();

    for (i, (strats, names)) in strategy_chunks.iter().zip(name_chunks.iter()).enumerate() {
        let chunk_name = format_ident!("chunk_{}", i);
        chunk_names.push(chunk_name.clone());

        let strat_list: Vec<_> = strats.iter().collect();
        chunk_strategies.push(quote! { (#(#strat_list),*) });

        let name_list: Vec<_> = names.iter().collect();
        flatten_code.push(quote! {
            let (#(#name_list),*) = #chunk_name;
        });
    }

    quote! {
        #[cfg(any(test, feature = "fuzz"))]
        impl #pack_name {
            /// Generate a proptest strategy that produces semantically valid payload bytes.
            /// Enum-mapped fields will only contain valid enum values.
            pub fn fuzz_payload_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::prelude::*;
                (
                    #(#chunk_strategies),*
                ).prop_map(|(#(#chunk_names),*)| {
                    #(#flatten_code)*
                    let mut payload = Vec::with_capacity(#payload_len);
                    #(#field_serializers)*
                    payload
                })
            }

            /// Generate a proptest strategy that produces valid UBX frames with semantic validity.
            pub fn fuzz_frame_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::strategy::Strategy;
                Self::fuzz_payload_strategy().prop_map(|payload| {
                    crate::ubx_packets::fuzz_helpers::build_ubx_frame(#class, #id, &payload)
                })
            }

            /// Generate a proptest strategy that produces payload bytes with arbitrary values.
            /// For large packets, this generates random bytes of the correct length.
            pub fn fuzz_payload_chaos_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::prelude::*;
                proptest::collection::vec(any::<u8>(), #payload_len)
            }

            /// Generate a proptest strategy that produces UBX frames with arbitrary field values.
            pub fn fuzz_frame_chaos_strategy() -> impl proptest::strategy::Strategy<Value = Vec<u8>> {
                use proptest::strategy::Strategy;
                Self::fuzz_payload_chaos_strategy().prop_map(|payload| {
                    crate::ubx_packets::fuzz_helpers::build_ubx_frame(#class, #id, &payload)
                })
            }
        }
    }
}

/// Generate a strategy for a field, considering its map_type if present.
/// If the field has a map_type that looks like a simple enum (created with #[ubx_extend]),
/// use its valid values via UbxEnumFuzzable trait.
fn generate_strategy_for_field(field: &PackField) -> TokenStream {
    // Check if field has a map_type (enum mapping)
    if let Some(ref map_desc) = field.map.map_type {
        // Only use enum-aware strategy for simple types without generics/lifetimes
        // Types like CfgValIter<'a> or struct wrappers won't have UbxEnumFuzzable
        if is_simple_enum_type(&map_desc.ty) {
            let mapped_ty = &map_desc.ty;
            let raw_ty = &field.ty;
            
            // Generate a strategy that selects from valid enum values
            // Use prop_flat_map to handle the Index type from select()
            return quote! {
                {
                    let values = <#mapped_ty as crate::ubx_packets::fuzz_traits::UbxEnumFuzzable>::valid_raw_values();
                    (0..values.len()).prop_map(move |i| values[i] as #raw_ty)
                }
            };
        }
    }
    
    // No map_type or complex type, use default strategy based on raw type
    generate_strategy_for_type(&field.ty)
}

/// Check if a type looks like a simple enum created with #[ubx_extend].
/// We use a conservative whitelist approach - only known enum patterns get the trait.
fn is_simple_enum_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            // Must have no generic arguments
            if !segment.arguments.is_empty() {
                return false;
            }
            
            let name = segment.ident.to_string();
            
            // Exclude primitive types
            if matches!(name.as_str(), "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" | "bool") {
                return false;
            }
            
            // Exclude common non-enum suffixes (structs, bitflags, iterators)
            if name.ends_with("Flags") 
                || name.ends_with("Iter") 
                || name.ends_with("Status")
                || name.ends_with("Error")
                || name.ends_with("Info")
                || name.ends_with("Data")
                || name.ends_with("Config")
                || name.ends_with("Settings")
            {
                return false;
            }
            
            // Whitelist: types ending with "Type" or "Mode" are typically enums
            // Also include types with "Fix" in the name (like GnssFixType)
            if name.ends_with("Type") 
                || name.ends_with("Mode") 
                || name.ends_with("Source")
                || name.ends_with("Power")
                || name.contains("Fix")
            {
                return true;
            }
            
            // Default: don't assume it's an enum - safer to use any::<T>()
            return false;
        }
    }
    false
}

fn generate_strategy_for_type(ty: &syn::Type) -> TokenStream {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "u8" => quote! { any::<u8>() },
                "i8" => quote! { any::<i8>() },
                "u16" => quote! { any::<u16>() },
                "i16" => quote! { any::<i16>() },
                "u32" => quote! { any::<u32>() },
                "i32" => quote! { any::<i32>() },
                "u64" => quote! { any::<u64>() },
                "i64" => quote! { any::<i64>() },
                "f32" => quote! { any::<f32>() },
                "f64" => quote! { any::<f64>() },
                _ => quote! { any::<u8>() }, // Fallback for unknown types
            }
        }
        syn::Type::Array(array) => {
            // Handle fixed-size arrays like [u8; N]
            let len = &array.len;
            let elem_ty = &array.elem;
            if is_u8_type(elem_ty) {
                quote! { proptest::collection::vec(any::<u8>(), #len).prop_map(|v| {
                    let mut arr = [0u8; #len];
                    arr.copy_from_slice(&v);
                    arr
                }) }
            } else {
                // For other array types, generate element strategies
                quote! { any::<[u8; #len]>() }
            }
        }
        _ => quote! { any::<u8>() }, // Fallback
    }
}

fn generate_serializer_for_type(ty: &syn::Type, field_name: &syn::Ident) -> TokenStream {
    match ty {
        syn::Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            let type_name = segment.ident.to_string();
            match type_name.as_str() {
                "u8" | "i8" => quote! { payload.push(#field_name as u8); },
                "u16" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "i16" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "u32" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "i32" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "u64" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "i64" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "f32" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                "f64" => quote! { payload.extend_from_slice(&#field_name.to_le_bytes()); },
                _ => quote! { payload.push(#field_name as u8); }, // Fallback
            }
        }
        syn::Type::Array(_) => {
            quote! { payload.extend_from_slice(&#field_name); }
        }
        _ => quote! { payload.push(#field_name as u8); }, // Fallback
    }
}

fn is_u8_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "u8";
        }
    }
    false
}

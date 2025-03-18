use alloc::collections::{BTreeMap as HashMap, BTreeSet as HashSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, LitStr};
use proc_macro2::TokenStream as TokenStream2;

pub(crate) const BYTE: &str = "byte";

#[derive(Clone)]
pub(crate) struct Field {
    pub name: String,
    pub type_name: String,  // Called 'type' in TS but that's a keyword in Rust
}

#[derive(Clone)]
pub(crate) enum MolType {
    Array {
        name: String,
        item: String,
        item_count: usize,
    },
    Vector {
        name: String,
        item: String,
    },
    Option {
        name: String,
        item: String,
    },
    Union {
        name: String,
        items: Vec<UnionItem>,
    },
    Struct {
        name: String,
        fields: Vec<Field>,
    },
    Table {
        name: String,
        fields: Vec<Field>,
    },
}

#[derive(Clone)]
pub(crate) enum UnionItem {
    Type(String),
    TypeWithId(String, u32),
}

pub(crate) struct CodecMap {
    types: Vec<MolType>,
}

impl Parse for CodecMap {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content = input.parse::<LitStr>()?.value();
        let types = parse_schema(&content);
        validate_parser_results(&types)?;
        
        // Sort types in dependency order
        let sorted_types = topological_sort(&types)?;
        
        Ok(CodecMap { types: sorted_types })
    }
}

impl CodecMap {
    pub(crate) fn parse_schema(content: &str) -> syn::Result<Self> {
        let types = parse_schema(content);
        validate_parser_results(&types)?;
        let sorted_types = topological_sort(&types)?;
        Ok(CodecMap { types: sorted_types })
    }
}

fn validate_parser_results(results: &[MolType]) -> syn::Result<()> {
    check_duplicate_names(results)?;
    check_dependencies(results)?;
    Ok(())
}

fn check_duplicate_names(results: &[MolType]) -> syn::Result<()> {
    let mut names = HashSet::new();

    for mol_type in results {
        let name = match mol_type {
            MolType::Array { name, .. } |
            MolType::Vector { name, .. } |
            MolType::Option { name, .. } |
            MolType::Union { name, .. } |
            MolType::Struct { name, .. } |
            MolType::Table { name, .. } => name,
        };

        if !names.insert(name) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Duplicate name: {}", name)
            ));
        }

        // Check duplicate field names in struct and table
        if let MolType::Struct { fields, .. } | MolType::Table { fields, .. } = mol_type {
            let mut field_names = HashSet::new();
            for field in fields {
                if !field_names.insert(&field.name) {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Duplicate field name: {}", field.name)
                    ));
                }
            }
        }
    }
    Ok(())
}

fn assert_fixed_mol_type(type_name: &str, map: &MolTypeMap) -> syn::Result<()> {
    if type_name == BYTE {
        return Ok(());
    }
    
    let mol_type = map.get(type_name).ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Type not found: {}", type_name)
        )
    })?;

    match mol_type {
        MolType::Array { item, .. } => {
            if item != BYTE {
                assert_fixed_mol_type(item, map)?;
            }
        }
        MolType::Struct { fields, .. } => {
            for field in fields {
                if field.type_name != BYTE {
                    assert_fixed_mol_type(&field.type_name, map)?;
                }
            }
        }
        _ => {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Type {} should be fixed length", type_name)
            ))
        }
    }
    Ok(())
}

type MolTypeMap = HashMap<String, MolType>;

fn to_mol_type_map(types: &[MolType]) -> MolTypeMap {
    types.iter()
        .map(|t| {
            let name = match t {
                MolType::Array { name, .. } |
                MolType::Vector { name, .. } |
                MolType::Option { name, .. } |
                MolType::Union { name, .. } |
                MolType::Struct { name, .. } |
                MolType::Table { name, .. } => name.clone(),
            };
            (name, t.clone())
        })
        .collect()
}

fn check_dependencies(results: &[MolType]) -> syn::Result<()> {
    let map = to_mol_type_map(results);

    for mol_type in results {
        match mol_type {
            MolType::Array { name, item, .. } => {
                if item != BYTE {
                    assert_fixed_mol_type(item, &map)?;
                }
            }
            MolType::Struct { name, fields, .. } => {
                for field in fields {
                    if field.type_name != BYTE {
                        assert_fixed_mol_type(&field.type_name, &map)?;
                    }
                }
            }
            MolType::Vector { item, .. } | MolType::Option { item, .. } => {
                if item != BYTE && !map.contains_key(item) {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Undefined type: {}", item)
                    ));
                }
            }
            MolType::Union { items, .. } => {
                for item in items {
                    match item {
                        UnionItem::Type(type_name) => {
                            if type_name != BYTE && !map.contains_key(type_name) {
                                return Err(syn::Error::new(
                                    proc_macro2::Span::call_site(),
                                    format!("Undefined union type: {}", type_name)
                                ));
                            }
                        }
                        UnionItem::TypeWithId(type_name, id) => {
                            if type_name != BYTE && !map.contains_key(type_name) {
                                return Err(syn::Error::new(
                                    proc_macro2::Span::call_site(),
                                    format!("Undefined union type: {}", type_name)
                                ));
                            }
                            // Validate id is a valid u32 (already done at parsing time)
                        }
                    }
                }
            }
            MolType::Table { fields, .. } => {
                for field in fields {
                    if field.type_name != BYTE && !map.contains_key(&field.type_name) {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            format!("Undefined field type: {}", field.type_name)
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

// Topological sort to ensure dependencies are properly ordered
fn topological_sort(types: &[MolType]) -> syn::Result<Vec<MolType>> {
    let map = to_mol_type_map(types);
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut temp_mark = HashSet::new();

    fn visit(
        name: &str,
        map: &MolTypeMap,
        result: &mut Vec<MolType>,
        visited: &mut HashSet<String>,
        temp_mark: &mut HashSet<String>,
    ) -> syn::Result<()> {
        if temp_mark.contains(name) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("Circular dependency detected involving type: {}", name),
            ));
        }
        
        if visited.contains(name) {
            return Ok(());
        }
        
        if let Some(mol_type) = map.get(name) {
            temp_mark.insert(name.to_string());
            
            // Visit dependencies first
            match mol_type {
                MolType::Array { item, .. } | MolType::Vector { item, .. } | MolType::Option { item, .. } => {
                    if item != BYTE {
                        visit(item, map, result, visited, temp_mark)?;
                    }
                }
                MolType::Struct { fields, .. } | MolType::Table { fields, .. } => {
                    for field in fields {
                        if field.type_name != BYTE {
                            visit(&field.type_name, map, result, visited, temp_mark)?;
                        }
                    }
                }
                MolType::Union { items, .. } => {
                    for item in items {
                        let type_name = match item {
                            UnionItem::Type(name) => name,
                            UnionItem::TypeWithId(name, _) => name,
                        };
                        if type_name != BYTE {
                            visit(type_name, map, result, visited, temp_mark)?;
                        }
                    }
                }
            }
            
            temp_mark.remove(name);
            visited.insert(name.to_string());
            result.push(mol_type.clone());
        }
        
        Ok(())
    }
    
    for mol_type in types {
        let name = match mol_type {
            MolType::Array { name, .. } |
            MolType::Vector { name, .. } |
            MolType::Option { name, .. } |
            MolType::Union { name, .. } |
            MolType::Struct { name, .. } |
            MolType::Table { name, .. } => name,
        };
        
        if !visited.contains(name) {
            visit(name, &map, &mut result, &mut visited, &mut temp_mark)?;
        }
    }
    
    Ok(result)
}

fn parse_schema(content: &str) -> Vec<MolType> {
    let mut types = Vec::new();
    let mut current_block = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        if line.ends_with('{') {
            current_block.push(line);
        } else if line == "}" && !current_block.is_empty() {
            current_block.push(line);
            if let Some(mol_type) = parse_block(&current_block) {
                types.push(mol_type);
            }
            current_block.clear();
        } else if line.ends_with(';') {
            // Single line type definitions
            if let Some(mol_type) = parse_single_line(line) {
                types.push(mol_type);
            }
        } else if !current_block.is_empty() {
            current_block.push(line);
        }
    }

    types
}

fn parse_single_line(line: &str) -> Option<MolType> {
    let line = line.trim_end_matches(';');
    let parts: Vec<&str> = line.split_whitespace().collect();

    match parts.get(0)? {
        &"vector" => {
            let name = parts.get(1)?.to_string();
            let item = parts.get(3)?.trim_matches('<').trim_matches('>').to_string();
            Some(MolType::Vector { name, item })
        },
        &"array" => {
            let name = parts.get(1)?.to_string();
            let item = parts.get(3)?.trim_matches('[').to_string();
            let size = parts.get(5)?.parse().ok()?;
            Some(MolType::Array { name, item, item_count: size })
        },
        &"option" => {
            let name = parts.get(1)?.to_string();
            let item = parts.get(3)?.trim_matches('(').trim_matches(')').to_string();
            Some(MolType::Option { name, item })
        },
        &"union" => {
            let name = parts.get(1)?.to_string();
            let items_str = parts.get(3)?;
            let items = items_str
                .trim_matches('<')
                .trim_matches('>')
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        return None;
                    }
                    if s.contains('(') {
                        let parts: Vec<&str> = s
                            .trim_matches('(')
                            .trim_matches(')')
                            .split(',')
                            .map(|s| s.trim())
                            .collect();
                        let type_name = parts.get(0)?.to_string();
                        let id = parts.get(1)?.parse().ok()?;
                        Some(UnionItem::TypeWithId(type_name, id))
                    } else {
                        Some(UnionItem::Type(s.to_string()))
                    }
                })
                .collect();
            Some(MolType::Union { name, items })
        },
        _ => None,
    }
}

fn parse_block(lines: &[&str]) -> Option<MolType> {
    if lines.is_empty() {
        return None;
    }

    let first_line = lines[0];
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let keyword = parts.get(0)?;

    match *keyword {
        "table" | "struct" => {
            let name = parts.get(1)?.trim_end_matches('{').to_string();
            let mut fields = Vec::new();
            
            // Parse fields
            for i in 1..lines.len() {
                let line = lines[i].trim();
                
                // Skip braces and empty lines
                if line == "}" || line.is_empty() {
                    continue;
                }
                
                // Remove trailing comma if present
                let line = line.trim_end_matches(',');
                
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    fields.push(Field {
                        name: parts[0].trim().to_string(),
                        type_name: parts[1].trim().to_string(),
                    });
                }
            }

            match *keyword {
                "table" => Some(MolType::Table { name, fields }),
                "struct" => Some(MolType::Struct { name, fields }),
                _ => None,
            }
        },
        "union" => {
            let name = parts.get(1)?.trim_end_matches('{').to_string();
            let mut items = Vec::new();

            // Parse union items
            for i in 1..lines.len() {
                let line = lines[i].trim();
                
                // Skip braces and empty lines
                if line == "}" || line.is_empty() {
                    continue;
                }
                
                // Remove trailing comma if present
                let line = line.trim_end_matches(',');
                
                if line.contains(':') {
                    // Item with ID
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        let type_name = parts[0].trim().to_string();
                        if let Ok(id) = parts[1].trim().parse::<u32>() {
                            items.push(UnionItem::TypeWithId(type_name, id));
                        }
                    }
                } else {
                    // Simple item
                    items.push(UnionItem::Type(line.to_string()));
                }
            }

            Some(MolType::Union { name, items })
        },
        _ => None,
    }
}

pub(crate) fn generate_molecule_code(schema: CodecMap, map_name: &syn::Ident) -> TokenStream {
    // Generate type definitions
    let type_definitions = generate_type_definitions(&schema.types);
    
    // Generate CodecMap implementation
    let codec_map = generate_codec_map(&schema.types);

    TokenStream::from(quote! {
        #type_definitions

        pub struct #map_name;

        impl #map_name {
            pub fn get_instance() -> &'static Self {
                static INSTANCE: #map_name = #map_name;
                &INSTANCE
            }

            #codec_map
        }
    })
}

fn generate_type_definitions(types: &[MolType]) -> TokenStream2 {
    let expanded = types.iter().map(|mol_type| {
        match mol_type {
            MolType::Table { name, fields } => {
                let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let field_defs = fields.iter().map(|field| {
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    let field_type = syn::Ident::new(&field.type_name, proc_macro2::Span::call_site());
                    quote! {
                        pub #field_name: #field_type,
                    }
                });
                
                quote! {
                    #[derive(Serialize, Deserialize, Clone, Debug)]
                    pub struct #name_ident {
                        #(#field_defs)*
                    }
                }
            },
            MolType::Struct { name, fields } => {
                let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let field_defs = fields.iter().map(|field| {
                    let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
                    let field_type = syn::Ident::new(&field.type_name, proc_macro2::Span::call_site());
                    quote! {
                        pub #field_name: #field_type,
                    }
                });
                
                quote! {
                    #[derive(Clone, Debug)]
                    pub struct #name_ident {
                        #(#field_defs)*
                    }
                }
            },
            MolType::Vector { name, item } => {
                let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let item_ident = syn::Ident::new(item, proc_macro2::Span::call_site());
                quote! {
                    pub type #name_ident = Vec<#item_ident>;
                }
            },
            MolType::Array { name, item, item_count } => {
                let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let item_ident = syn::Ident::new(item, proc_macro2::Span::call_site());
                quote! {
                    pub type #name_ident = [#item_ident; #item_count];
                }
            },
            MolType::Option { name, item } => {
                let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let item_ident = syn::Ident::new(item, proc_macro2::Span::call_site());
                quote! {
                    pub type #name_ident = Option<#item_ident>;
                }
            },
            MolType::Union { name, items } => {
                let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let item_defs = items.iter().map(|item| {
                    match item {
                        UnionItem::Type(type_name) => {
                            let type_ident = syn::Ident::new(type_name, proc_macro2::Span::call_site());
                            quote! { #type_ident }
                        },
                        UnionItem::TypeWithId(type_name, id) => {
                            let type_ident = syn::Ident::new(type_name, proc_macro2::Span::call_site());
                            quote! { (#type_ident, #id) }
                        },
                    }
                });
                
                quote! {
                    pub enum #name_ident {
                        #(#item_defs),*
                    }
                }
            },
        }
    });

    quote! {
        #(#expanded)*
    }
}

fn generate_codec_map(types: &[MolType]) -> TokenStream2 {
    let methods = types.iter().map(|mol_type| {
        let name = match mol_type {
            MolType::Array { name, .. } |
            MolType::Vector { name, .. } |
            MolType::Option { name, .. } |
            MolType::Union { name, .. } |
            MolType::Struct { name, .. } |
            MolType::Table { name, .. } => name,
        };
        let name_str = name.as_str();
        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());

        quote! {
            pub fn #name_ident(&self) -> &'static dyn molecule::Codec {
                static CODEC: molecule::Codec = molecule::Codec::new(#name_str);
                &CODEC
            }
        }
    });

    quote! {
        #(#methods)*
    }
} 

use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote,format_ident};
use syn;


#[proc_macro_derive(DeserializeBestEffort, attributes(serde))]
pub fn deserialize_best_effort_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_deserialize_best_effort_macro(&ast)
}

fn impl_deserialize_best_effort_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = get_struct_data(&ast);
    let fields = get_struct_fields(&ast);

    let field_enum_and_field_visitor = impl_field_enum_visitor(&fields);
    let struct_visitor = impl_struct_visitor(&data, name);
    let fields_array = get_fields_array(&fields);

    let visitor_name = get_visitor_name(&name);
    // Build impl
    let gen = quote! {
        impl<'de> DeserializeBestEffort<'de> for #name {}
        impl<'de> serde::de::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                #field_enum_and_field_visitor

                #struct_visitor

                #fields_array
                deserializer.deserialize_struct(stringify!(#name), FIELDS, #visitor_name)

            }
        }
    };
    gen.into()
}

fn get_alias_attrs(struct_fields: &syn::Field) -> Vec<String> {
    let mut lit_list = Vec::new();
    for attr in &struct_fields.attrs{
        let tokens = get_alias_attrs_variables(&attr);
        for val in tokens {
            let mut string_quote = "".to_string();
            if let syn::Lit::Str(string_lit) = val {
                string_quote = string_lit.value();
            }
            if string_quote.is_empty() {
                continue;
            }
            lit_list.push(string_quote);
        }
    }
    lit_list
}

fn get_alias_attrs_variables(attr: &syn::Attribute) -> Vec<syn::Lit> {
    let meta_items = get_serde_meta_items(attr).unwrap();
    let mut lit_list: Vec<syn::Lit> = Vec::new();
    for meta_item in meta_items{
        match meta_item {
            // Parse `#[serde(alias = "foo")]`
            syn::NestedMeta::Meta(syn::Meta::NameValue(m)) if m.path.is_ident("alias") => {
                lit_list.push(m.lit);
            },
            _ => (),
        };
    }
    lit_list
}

// From https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs line 1566
fn get_serde_meta_items(attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, ()> {
    if !attr.path.is_ident("serde") {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(syn::Meta::List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(_other) => {
            // cx.error_spanned_by(other, "expected #[serde(...)]");
            Err(())
        }
        Err(_err) => {
            // cx.syn_error(err);
            Err(())
        }
    }
}

fn get_fields_array(struct_fields: &[&syn::Field]) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for (i, field) in struct_fields.iter().enumerate(){
        // parse normal name (name of variable)
        // Ex: `pub name: String,` will add `"name",` to the list
        let ident = field.ident.as_ref().unwrap();
        let field_ident = get_enum_ident(ident,i);
        parse_gen = quote!{
            #parse_gen stringify!(#field_ident),
        };
        // parse alias names (if any)
        // Ex: `#[serde(alias = "type")]` will add `"type",` to the list
        let alias_ident = get_alias_attrs(field);
        for alias in alias_ident{
            parse_gen = quote!{
                #parse_gen stringify!(#alias),
            };
        }
    }
    parse_gen = quote!{
        const FIELDS: &'static [&'static str] = &[#parse_gen];
    };
    parse_gen
}

fn get_field_enum(struct_fields: &[&syn::Field]) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for (i, field) in struct_fields.iter().enumerate(){
        let ident = field.ident.as_ref().unwrap();
        let field_ident = get_enum_ident(ident,i);
        parse_gen = quote!{
            #parse_gen
            #field_ident,
        }
    }
    parse_gen = quote!{
        enum Field {
            #parse_gen
            Unknown(String),
        }
    };
    parse_gen
}
fn get_field_enum_match(struct_fields: &[&syn::Field]) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for(i, field) in struct_fields.iter().enumerate(){
        let ident = field.ident.as_ref().unwrap();
        let field_enum_ident = get_enum_ident(ident,i);
        parse_gen = quote!{
            #parse_gen
            // "id" => Ok(Field::Id),
            stringify!(#ident) => Ok(Field::#field_enum_ident),
        };
        // from #[serde(alias = "type")]
        let alias_ident = get_alias_attrs(field);


        for alias in alias_ident{
            parse_gen = quote!{
                #parse_gen
                // "type" => Ok(Field::Id),
                #alias => Ok(Field::#field_enum_ident),
            };
        }
    }
    parse_gen = quote!{
        match value {
            // "id" => Ok(Field::Id),
            #parse_gen
            _ => Ok(Field::Unknown(value.to_string())),
        }
    };
    parse_gen
}

fn impl_field_enum_visitor(struct_fields: &[&syn::Field]) -> proc_macro2::TokenStream{
    let field_enum = get_field_enum(&struct_fields);
    let field_enum_match = get_field_enum_match(&struct_fields);

    let parse_gen = quote!{
        // enum Field { Id, Name, Race, Unknown, Special };
        #field_enum

        impl<'de> serde::de::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("Did not expect this... se default is not working.")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        #field_enum_match
                    }
                }
                // deserialize_any
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }
    };
    parse_gen
}


fn impl_struct_visitor(struct_data: &[(&syn::Ident, &syn::Type)], name: &syn::Ident) -> proc_macro2::TokenStream{
    let visit_seq = get_struct_visit_seq(&struct_data, &name);
    let visit_map = get_struct_visit_map(&struct_data, &name);

    let visitor_name = get_visitor_name(&name);

    let parse_gen = quote!{
        struct #visitor_name;

        impl<'de> serde::de::Visitor<'de> for #visitor_name {
            type Value = #name;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(&format!("struct {}",stringify!(#visitor_name)))
            }

            #visit_seq

            #visit_map

        }

    };
    parse_gen
}

fn get_struct_visit_seq(struct_data: &[(&syn::Ident, &syn::Type)], name: &syn::Ident) -> proc_macro2::TokenStream{

    let variable_init = set_struct_visit_seq_variable(&struct_data);
    let create_object = set_struct_create_object(&struct_data, &name);

    // This code is not used for structs
    // TODO: Test this code if it works
    let parse_gen = quote!{
        fn visit_seq<V>(self, mut seq: V) -> Result<#name, V::Error>
        where
            V: serde::de::SeqAccess<'de>,
        {
            // not used for struct it seems
            #variable_init
            Ok(#create_object)
        }

    };
    parse_gen
}

fn set_struct_visit_seq_variable(struct_data: &[(&syn::Ident, &syn::Type)]) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for (i, (field,type_)) in struct_data.iter().enumerate(){
        parse_gen = quote!{
            #parse_gen
            // Example of generated code here:
            // let id = seq.next_element()?
            //     .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let #field:#type_ = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(#i, &self))?;
        };
    }
    parse_gen
}

fn get_struct_visit_map(struct_data: &[(&syn::Ident, &syn::Type)], name: &syn::Ident) -> proc_macro2::TokenStream{

    let variable_init = set_struct_visit_map_variable(&struct_data);
    let enum_match_variable = set_struct_visit_map_enum_match(&struct_data);
    let create_object = set_struct_create_object(&struct_data, &name);

    let parse_gen = quote!{
        fn visit_map<V>(self, mut map: V) -> Result<#name, V::Error>
        where
            V: serde::de::MapAccess<'de>,
        {

            #variable_init

            while let Some(key) = map.next_key()? {

                #enum_match_variable
            }
            Ok(#create_object)
        }

    };
    parse_gen

}

fn set_struct_create_object(struct_data: &[(&syn::Ident, &syn::Type)], name: &syn::Ident) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for (field,_type) in struct_data{
        parse_gen = quote!{
            #parse_gen
            //id,
            // similar to `id: id,`
            #field,
        }
    }
    parse_gen = quote!{
        //#[allow(clippy::needless_update)]
        #name{
            #parse_gen
            //..Default::default()
        }
    };
    parse_gen
}

fn set_struct_visit_map_variable(struct_data: &[(&syn::Ident, &syn::Type)]) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for (field,type_) in struct_data{
        parse_gen = quote!{
            #parse_gen
            // Example of generated code here:
            //let mut id:i32 = Default::default();
            let mut #field:#type_ = Default::default();
        }
    }
    parse_gen
}

fn set_struct_visit_map_enum_match(struct_data: &[(&syn::Ident, &syn::Type)]) -> proc_macro2::TokenStream{
    let mut parse_gen = quote!{};
    for (i, (field,_type)) in struct_data.iter().enumerate(){
        let field_ident = get_enum_ident(field, i);
        parse_gen = quote!{
            #parse_gen
            // Example of generated code here:
            // Field::Enum_id => {
            //     let next_value = map.next_value().unwrap_or_default();
            //     id.add_data(next_value);
            // }
            Field::#field_ident => {
                let next_value = map.next_value().unwrap_or_default();
                #field.add_data(stringify!(#field), next_value);
            }
        }
    }
    parse_gen = quote!{
        match key {
            #parse_gen
            Field::Unknown(key_name) => {
                let next_value = map.next_value().unwrap_or_default();
                unknown.add_data(&key_name, next_value);
            }
        }
    };
    parse_gen
}

fn get_enum_ident(_ident: &syn::Ident, nummer: usize) -> syn::Ident{
    format_ident!("Enum{}", nummer)
}

fn get_visitor_name(ident: &syn::Ident) -> syn::Ident{
    format_ident!("Struct{}Visitor", ident)
}

/// Get a list of all the items in the Struct
/// Example
/// ```ignore
/// Point{
///     x: i32,
///     y: i32,
/// }
/// ```
/// Will return `["x","y"]` but then as an `syn::Ident`
fn get_struct_data(ast: &syn::DeriveInput)-> Vec<(&syn::Ident, &syn::Type)> {
    let mut list = Vec::new();
    match &ast.data{
        syn::Data::Struct(x) => {
            match &x.fields {
                syn::Fields::Named(x) => {
                    for field in &x.named {
                        list.push((field.ident.as_ref().unwrap(), &field.ty));
                    }
                    Some(())
                },
                syn::Fields::Unnamed(_) => None,
                syn::Fields::Unit => None,
            }
        },
        syn::Data::Enum(_x) => None,
        syn::Data::Union(_x) => None,
    };
    list
}

fn get_struct_fields(ast: &syn::DeriveInput)-> Vec<&syn::Field> {
    let mut list = Vec::new();
    match &ast.data{
        syn::Data::Struct(x) => {
            match &x.fields {
                syn::Fields::Named(x) => {
                    for field in &x.named {
                        list.push(field);
                    }
                    Some(())
                },
                syn::Fields::Unnamed(_) => None,
                syn::Fields::Unit => None,
            }
        },
        syn::Data::Enum(_x) => None,
        syn::Data::Union(_x) => None,
    };
    list
}

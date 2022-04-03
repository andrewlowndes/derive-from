use std::{collections::HashMap};
use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Token, punctuated::Punctuated, Path, Lit};
use syn_unnamed_struct::{Meta, MetaValue, NestedMeta};

const INTO_METHOD: &str = "core::convert::Into::into";
const SKIP_METHOD: &str = "core::default::Default::default()";

#[derive(Debug, Default)]
struct FieldOverridesArgs {
    pub map: Option<String>,
    pub rename: Option<String>,
    pub skip: bool,
    pub default: Option<String>,
}

#[derive(Debug, Default)]
struct FieldArgs {
    pub map: Option<String>,
    pub rename: Option<String>,
    pub skip: bool,
    pub default: Option<String>,
    pub overrides: HashMap<Ident, FieldOverridesArgs>,
}

fn extract_meta_str(expr: &MetaValue) -> String {
    match expr {
        MetaValue::Lit(value) => {
            match &value {
                Lit::Str(value_str) => value_str.value(),
                _ => panic!("Only strings supported: {}", value.to_token_stream())
            }
        },
        _ => panic!("Expected literal key name: {}", expr.into_token_stream())
    }
}

fn extract_meta_bool(expr: &MetaValue) -> bool {
    match expr {
        MetaValue::Lit(value) => {
            match &value {
                Lit::Bool(value_bool) => value_bool.value(),
                _ => panic!("Only bools supported: {}", value.to_token_stream())
            }
        },
        _ => panic!("Expected literl key name: {}", expr.to_token_stream())
    }
}

fn extract_field_override_arg(custom_expr: &MetaValue) -> FieldOverridesArgs {
    let mut overrides = FieldOverridesArgs::default();
    
    match custom_expr {
        MetaValue::UnnamedMetaList(subobj) => {
            for field in &subobj.nested {
                match field {
                    NestedMeta::Meta(meta) => {
                        match meta {
                            Meta::Path(path) => {
                                match path.to_token_stream().to_string().as_str() {
                                    "skip" => {
                                        overrides.skip = true;
                                    },
                                    _ => panic!("Unrecognised bool property")
                                }
                            },
                            Meta::NameValue(pair) => {
                                let subobj_field_name = pair.path.to_token_stream().to_string();
                                
                                match subobj_field_name.as_str() {
                                    "map" => {
                                        overrides.map = Some(extract_meta_str(&pair.value));
                                    },
                                    "rename" => {
                                        overrides.rename = Some(extract_meta_str(&pair.value));
                                    },
                                    "default" => {
                                        overrides.default = Some(extract_meta_str(&pair.value));
                                    },
                                    "skip" => {
                                        overrides.skip = extract_meta_bool(&pair.value)
                                    },
                                    _ => panic!("Could not match override property: {}", subobj_field_name)
                                }
                            },
                            _ => panic!("Each override should be a name / value pair or single truthy value")
                        }
                    },
                    _ => panic!("Expects named fields in each override")
                }
            }
        },
        _ => panic!("Each override value should be an unamed struct")
    }
    
    overrides
}

#[proc_macro_derive(From, attributes(from))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput { ident, attrs, data, .. } = &input;

    //fetch all of the types from the struct attribute macros
    let type_names = attrs.iter().flat_map(|attr| {
        attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated).expect("Could not parse 'from' attribute")
    }).map(|path| {
        path.get_ident().unwrap().clone()
    }).collect::<Vec<_>>();

    let obj = match data {
        syn::Data::Struct(obj) => obj,
        _ => panic!("Only structs supported in From macro")
    };

    //determine the field-specific properties
    let field_objs = obj.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("Structs must contain named fields").clone();
        let mut props = FieldArgs::default();

        field.attrs.iter().flat_map(|attr| {
            attr.parse_args_with(<Punctuated<Meta, Token![,]>>::parse_terminated).expect("Could not parse 'from' attribute")
        }).for_each(|meta| {
            match meta {
                Meta::Path(path) => {
                    match path.to_token_stream().to_string().as_str() {
                        "skip" => {
                            props.skip = true;
                        },
                        _ => panic!("Unrecognised bool property")
                    }
                },
                Meta::NameValue(pair) => {
                    match pair.path.to_token_stream().to_string().as_str() {
                        "map" => {
                            props.map = Some(extract_meta_str(&pair.value))
                        },
                        "rename" => {
                            props.rename = Some(extract_meta_str(&pair.value))
                        },
                        "default" => {
                            props.default = Some(extract_meta_str(&pair.value))
                        },
                        "skip" => {
                            props.skip = extract_meta_bool(&pair.value)
                        },
                        "overrides" => {
                            match pair.value {
                                MetaValue::UnnamedMetaList(obj) => {
                                    for nested_field in obj.nested {
                                        match nested_field {
                                            NestedMeta::Meta(meta) => {
                                                match meta {
                                                    Meta::NameValue(pair) => {
                                                        let field_name = pair.path.get_inner().get_ident().unwrap().clone();
        
                                                        if !type_names.contains(&field_name) {
                                                            panic!("Type does not exist for override: {}", field_name);
                                                        }

                                                        let overrides = extract_field_override_arg(&pair.value);
                                                        props.overrides.insert(field_name, overrides);
                                                    },
                                                    _ => panic!("Each override should list a type and properties to override")
                                                }
                                            },
                                            _ => panic!("Each override should list a type and properties to override")
                                        }
                                    }
                                },
                                _ => panic!("Overrides must be an unnamed meta list")
                            }
                        },
                        _ => panic!("Unrecognised key value pair")
                    }
                },
                _ => panic!("Expected name value pair")
            }
        });

        (field_name, props)
    }).collect::<Vec<_>>();

    //determine how each field will map
    let mut output = proc_macro2::TokenStream::new();

    for type_name in type_names {
        let fields = field_objs.iter().map(|(field_name, field_obj)| {
            let mut map = field_obj.map.clone();
            let mut rename = field_obj.rename.clone();
            let mut skip = field_obj.skip;
            let mut default = field_obj.default.clone();

            //override type-specific and field-specific settings
            if let Some(type_override) = field_obj.overrides.get(&type_name) {
                if type_override.map.is_some() {
                    map = type_override.map.clone();
                }
                
                if type_override.rename.is_some() {
                    rename = type_override.rename.clone();
                }
                
                if type_override.default.is_some() {
                    default = type_override.default.clone();
                }
                
                if type_override.skip {
                    skip = type_override.skip;
                }
            }

            let target_value = {
                if skip {
                    let method_name: proc_macro2::TokenStream = default.unwrap_or_else(|| {
                        SKIP_METHOD.to_string()
                    }).parse().expect("Could not parse skip method");

                    quote!(#method_name)
                } else {
                    let target_name = rename.map(|name| {
                        Ident::new(&name, Span::call_site())
                    }).unwrap_or_else(|| {
                        field_name.clone()
                    });

                    let method_name: proc_macro2::TokenStream = map.unwrap_or_else(|| {
                        INTO_METHOD.to_string()
                    }).parse().expect("Could not parse map method");

                    quote!(#method_name(obj.#target_name))
                }
            };

            quote!(#field_name: #target_value)
        }).collect::<Vec<_>>();

        let type_impl = quote! {
            impl ::std::convert::From<#type_name> for #ident {
                fn from(obj: #type_name) -> Self {
                    Self {
                        #(#fields),*
                    }
                }
            }
        };

        output.extend(type_impl);
    }

    output.into()
}

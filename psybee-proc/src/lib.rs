extern crate proc_macro;
use quote::quote;

fn extract_path(ty: &syn::Type) -> syn::Path {
    match ty {
        syn::Type::Path(syn::TypePath { path, .. }) => path.clone(),
        _ => panic!("Only Path is supported"),
    }
}

#[proc_macro_derive(StimulusParams)]
pub fn derive_answer_fn(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // parse the input tokens into a syntax tree
    let input = syn::parse_macro_input!(item as syn::ItemStruct);
    // find all the fields in the struct
    let fields = match input.fields {
        // read the fields from the struct
        syn::Fields::Named(fields) => fields.named,
        _ => unimplemented!(),
    };
    // impl a function that returns the field names and types
    let input = &input.ident;
    let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

    // field_types can be Option<T> or T - we create field_types_T to get the T
    let ret_strs = field_types
        .iter()
        .map(|f| {
            // if the field is an Option, we get the type inside the Option
            let path = extract_path(f);
            if path.segments[0].ident.to_string() == "Option" {
                let t = if let syn::PathArguments::AngleBracketed(args) = &path.segments[0].arguments {
                    let arg = args.args.iter().next().unwrap();
                    if let syn::GenericArgument::Type(t) = arg {
                        let _a = extract_path(t).segments[0].ident.clone();
                        quote!(
                             match value {
                                 Some(value) => Some(StimulusParamValue::#_a(value)),
                                 _ => None,
                             }
                        )
                    } else {
                        panic!("Only Option<T> is supported");
                    }
                } else {
                    panic!("Only Option<T> is supported");
                };
                t
            } else {
                let _a = path.segments[0].ident.clone();
                quote!(
                    Some(StimulusParamValue::#_a(value))
                )
            }
        })
        .collect::<Vec<_>>();

    // field_types can be Option<T> or T - we create field_types_T to get the T
    let ret_strs2 = field_types
        .iter()
        .map(|f| {
            // if the field is an Option, we get the type inside the Option
            let path = extract_path(f);
            if path.segments[0].ident.to_string() == "Option" {
                let t = if let syn::PathArguments::AngleBracketed(args) = &path.segments[0].arguments {
                    let arg = args.args.iter().next().unwrap();
                    if let syn::GenericArgument::Type(t) = arg {
                        let _a = extract_path(t).segments[0].ident.clone();
                        quote!(
                              if let StimulusParamValue::#_a(value) = value {
                                 Some(value)
                             } else {
                                 panic!("Invalid type for fiel")
                             }
                        )
                    } else {
                        panic!("Only Option<T> is supported");
                    }
                } else {
                    panic!("Only Option<T> is supported");
                };
                t
            } else {
                let _a = path.segments[0].ident.clone();
                quote!(
                    if let StimulusParamValue::#_a(value) = value {
                        value
                    } else {
                        panic!("Invalid type for field")
                    }
                )
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
         impl StimulusParams for #input {
            fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
                match name {
                    #(stringify!(#field_names) => {
                        let value = self.#field_names.clone();
                        #ret_strs
                    } )*
                    _ => None,
                }
            }

            fn set_param(&mut self, name: &str, value: StimulusParamValue) {
                match name {
                    #(stringify!(#field_names) => {
                        let value = #ret_strs2;
                        self.#field_names = value;
                    } )*
                    _ => panic!("Invalid field name {}", name),
                }
            }

        }

    };

    // convert the expanded code back into tokens
    proc_macro::TokenStream::from(expanded)
}

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CallFn)]
pub fn derive_call_fn(input: TokenStream) -> TokenStream {
    // Parse the input (the enum on which we're deriving) into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = input.ident;

    // Ensure that we're deriving on an enum, not a struct or union
    let data_enum = match input.data {
        syn::Data::Enum(ref e) => e,
        _ => {
            return syn::Error::new_spanned(enum_ident, "#[derive(CallFn)] can only be applied to enums")
                .to_compile_error()
                .into();
        }
    };

    // For each variant, we’ll build a match arm that calls the closure `fun(...)`.
    //
    // This example just calls `fun(shape)` for each variant.
    let match_arms = data_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        quote! {
            Self::#variant_ident ( shape )
            | Self::#variant_ident( shape )
            => fun(shape)
        }
    });

    // Generate an impl block for the enum that adds `call_fn`.
    // We make `call_fn` take `self` by value here; you can change it to `&self` if needed.
    let expanded = quote! {
        impl #enum_ident {
            pub fn call_fn<A,B>(self, fun: impl FnOnce(A) -> B) -> B {
                match self {
                    #( #match_arms ),*
                }
            }
        }
    };

    expanded.into()
}

// a derive macro that implements FromPyObject for a simple enum (from a snake_case string)
// the enum must also implement strum::EnumString
#[proc_macro_derive(FromPyStr)]
pub fn derive_from_py_str(input: TokenStream) -> TokenStream {
    // Parse the input (the enum on which we're deriving) into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = input.ident;

    // Generate an impl block for the enum that adds `call_fn`.
    // We make `call_fn` take `self` by value here; you can change it to `&self` if needed.
    let expanded = quote! {



        impl<'py> FromPyObject<'py> for #enum_ident {
            fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
                use std::str::FromStr;
                let s = ob.extract::<String>()?;
                if let Ok(v) = #enum_ident::from_str(&s) {
                    Ok(v)
                } else {
                    panic!("Invalid value for {}: {}", stringify!(#enum_ident), s)
                    // Err(PyErr::new::<exceptions::ValueError, _>(format!("Invalid value for {}: {}", stringify!(#enum_ident), s)))
                }
            }
        }
    };

    expanded.into()
}

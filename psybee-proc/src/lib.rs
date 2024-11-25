extern crate proc_macro;
use quote::quote;

fn extract_path(ty: &syn::Type) -> syn::Path {
    match ty {
        syn::Type::Path(syn::TypePath { path, .. }) => {
            path.clone()
        }
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
    let ret_strs = field_types.iter().map(|f| {
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

    }).collect::<Vec<_>>();


    // field_types can be Option<T> or T - we create field_types_T to get the T
    let ret_strs2 = field_types.iter().map(|f| {
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

    }).collect::<Vec<_>>();




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

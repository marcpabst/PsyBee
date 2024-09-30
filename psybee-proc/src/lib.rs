extern crate proc_macro;
use quote::quote;

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

    // fields can have "default(x)", "min(x)", "max(x)" attributes
    //let field_attrs = fields.iter().map(|f| &f.attrs).collect::<Vec<_>>();

    let expanded = quote! {
        impl StimulusParams for #input {
            fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
                match name {
                    #(stringify!(#field_names) => {
                        let value = self.#field_names.clone();
                        Some(StimulusParamValue::#field_types(value))
                    } )*
                    _ => None,
                }
            }

            fn set_param(&mut self, name: &str, value: StimulusParamValue) {
                match name {
                    #(stringify!(#field_names) => {
                        match value {
                            StimulusParamValue::#field_types(value) => {
                                self.#field_names = value;
                            }
                            _ => panic!("Invalid type for field {}. Expected {:?} but got {:?}", name, stringify!(#field_types), value),
                        }
                    } )*
                    _ => panic!("Invalid field name {}", name),
                }
            }
        }

    };

    // convert the expanded code back into tokens
    proc_macro::TokenStream::from(expanded)
}

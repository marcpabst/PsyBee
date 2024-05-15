extern crate proc_macro;
use core::panic;

use itertools::izip;
use proc_macro::{TokenStream, TokenTree};
use quote::format_ident;
use quote::quote;

use quote::ToTokens;
use syn::parse::Parse;
use syn::Expr;
use syn::Path;
use syn::{
    parse_macro_input, punctuated::Punctuated, FnArg, Ident, ItemFn, Signature, Token,
    Type,
};

static NEVER_WRAP: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
    "bool", "char", "String", "str",
];

fn parse_sigature(
    input: Signature,
) -> (
    Ident,
    Vec<Ident>,
    Vec<Ident>,
    Vec<bool>,
    Vec<bool>,
    proc_macro2::TokenStream,
    bool,
    bool,
) {
    let fn_name = &input.ident;
    let fn_receiver = &input.receiver();

    let is_static = match fn_receiver {
        Some(_) => false,
        None => true,
    };

    let mut fn_args_names = Vec::new();
    let mut fn_args_types = Vec::new();
    let mut fn_args_mut = Vec::new();
    let mut fn_args_refs = Vec::new();

    for (i, input) in input.inputs.iter().enumerate() {
        match input {
            syn::FnArg::Receiver(_) => (),
            syn::FnArg::Typed(pat_type) => {
                // the arg name
                match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) => {
                        fn_args_names.push(pat_ident.ident.clone());
                    }
                    _ => panic!("Expected ident"),
                }
                // the arg type as an expression (e.g. "u32" or "&mut u32")
                match &*pat_type.ty {
                    Type::Reference(type_ref) => {
                        fn_args_types.push(match &*type_ref.elem {
                            Type::Path(type_path) => {
                                fn_args_refs.push(true);
                                // if the argument is a mutable reference, we need to unwrap it
                                match type_ref.mutability {
                                    Some(_) => {
                                        fn_args_mut.push(true);
                                        type_path.path.segments[0].ident.clone()
                                    }
                                    None => {
                                        fn_args_mut.push(false);
                                        type_path.path.segments[0].ident.clone()
                                    }
                                }
                            }
                            _ => panic!("Expected path"),
                        });
                    }
                    Type::Path(type_path) => {
                        fn_args_types.push(type_path.path.segments[0].ident.clone());
                        fn_args_mut.push(false);
                        fn_args_refs.push(false);
                    }
                    _ => panic!("Expected path"),
                }
            }
            _ => {}
        }
    }
    // the return type
    // check if the function has a return type
    let is_void = match &input.output {
        // if it doesn't have a return type, it's void
        syn::ReturnType::Default => true,
        // if the return type is "()" it's void
        syn::ReturnType::Type(_, ty) => match &**ty {
            Type::Tuple(tuple) => tuple.elems.is_empty(),
            _ => false,
        },
        _ => false,
    };

    let return_type = input.output.to_token_stream();

    // remove the first token
    let return_type = return_type.clone().into_iter().skip(2).collect();

    return (
        fn_name.clone(),
        fn_args_names,
        fn_args_types,
        fn_args_mut,
        fn_args_refs,
        return_type,
        is_void,
        is_static,
    );
}

/// ussage:
/// ```rust
/// unsafe_transmute_ignore_size!(A, B);
/// ```
///
/// This will generate the following code:
/// ```rust
/// {
/// let b = ::core::ptr::read(&a as *const A as *const B);
/// ::core::mem::forget(a);
/// b
/// }
#[proc_macro]
pub fn transmute_ignore_size(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as Expr);

    // Generate the new function body using unsafe code
    let expanded = quote! {
        {
            let a = #input;
            unsafe {
                let b = ::core::ptr::read(&a as *const _ as *const _);
                ::core::mem::forget(a);
                b
            }
        }
    };

    // Convert the expanded code back into a token stream and return it
    TokenStream::from(expanded)
}

/// This macro wraps the gien struct "Name" in newtype pattern "PyName"
/// and implements the Into trait for it.
///
/// Example:
/// ```rust
/// py_wrap!(SomeStruct);
/// ```
///
/// This will generate the following code:
/// ```rust
/// pub struct PySomeStruct(pub SomeStruct);
/// impl Into<SomeStruct> for PySomeStruct {
/// ...
/// }
/// ```
#[proc_macro]
pub fn py_wrap(s: TokenStream) -> TokenStream {
    // split into struct name and any additional arguments
    let s = proc_macro2::TokenStream::from(s);
    let split = split_token_stream_by_comma(s);

    // get the struct name as an ident
    let name = split
        .get(0)
        .expect("Expected struct name as first argument")
        .clone();

    let name: Ident = syn::parse2(name).unwrap();

    let additional_args = match split.get(1) {
        Some(args) => add_prefix_to_first_token(", ", args.clone()),
        None => proc_macro2::TokenStream::new(),
    };

    let py_name = syn::Ident::new(&format!("Py{}", name), name.span());

    let dumm_trait1_name = format_ident!("DummyTrait1{}", name);

    let exported_name = format!("{}", name);

    let expanded = quote::quote! {
        // make sure transmute_ignore_size! is in scope


        #[pyo3::prelude::pyclass(name= #exported_name #additional_args)]
        #[derive(Debug)]
        pub struct #py_name(pub #name);

        // implement the Into trait for the newtype pattern
        impl Into<#name> for #py_name {
            fn into(self) -> #name {
                self.0
            }
        }

        // implement the From trait for the newtype pattern
        impl From<#name> for #py_name {
            fn from(inner: #name) -> Self {
                Self(inner)
            }
        }

        // impl __str__
        #[pyo3::prelude::pymethods]
        impl #py_name {
            fn __str__(&self) -> String {
                format!("{:?}", self.0)
            }
        }

        // create dummy trait that is only implemented by our type
        pub trait #dumm_trait1_name {}
        impl #dumm_trait1_name for #name {}

        impl<T> From<&T> for #py_name
        where
            T: #dumm_trait1_name + Clone
        {
            fn from(inner: &T) -> Self {
                // this is an unsafe operation, but we know that inner is of type name
                // let's unsafely cast it to the inner type
                let inner = inner.clone();
                let inner = transmute_ignore_size!(inner);
                Self(inner)
            }
        }
    };

    TokenStream::from(expanded)
}

/// This macto implements a trait

// Method to add a prefix to the first token in a proc_macro2::TokenStream
fn add_prefix_to_first_token(
    prefix: &str,
    input: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    // we need to split the input into tokens
    let mut tokens = input.into_iter();
    // the first token must be an ident
    let first_token = tokens.next().unwrap().to_string();
    // we add the prefix to the first token
    let prefixed_token = format!("{}{}", prefix, first_token);
    // we create a new token stream with the prefixed token
    let prefixed_token_stream =
        syn::parse_str::<proc_macro2::TokenStream>(&prefixed_token).unwrap();
    // we add the rest of the tokens
    let rest = tokens.collect::<proc_macro2::TokenStream>();
    // we concatenate the two token streams
    let result = quote::quote! {#prefixed_token_stream #rest};
    result
}

// This macro takes a struct (for wich it assumes a wrapping struct PyName exists)
// and a method name, and implements the method for PyName, forwarding the call to the inner struct.
// It then scans the method signature for any arguments that are not supported by pyo3 and assumes that
// a wrapper exists for them. It then forwards the call to the inner struct, unwrapping the arguments
// and wrapping the return value.
//
// This macro supports methods with multiple arguments and return values. It also supports methods that
// need to be annotated with #[staticmethod]. If the provided method is a field, it will generate a getter
// and setter for it.
//
// It does not support methods with lifetimes or generics.
//
// Example:
// ```rust
// py_forward!(SomeStruct, fn some_method(&self, arg1: Arg1, arg2: Arg2) -> Ret);
// ```
//
// This will generate the following code:
// ```rust
// #[pymethods]
// impl PySomeStruct {
//     pub fn some_method(&self, arg1: PyArg1, arg2: PyArg2) -> PyRet {
//        PyRet(self.0.some_method(arg1.into(), arg2.into()).into())
//     }
// }
// ```
#[proc_macro]
pub fn py_forward(input: TokenStream) -> TokenStream {
    // convert the input to a token stream
    let input = proc_macro2::TokenStream::from(input);

    // we first split the input into the receiver and the method signature (separated by the first comma)
    let splitted = split_token_stream_by_comma(input);

    let receiver = splitted
        .get(0)
        .expect("Expected receiver as first argument")
        .clone();

    let method = splitted
        .get(1)
        .expect("Expected method as second argument")
        .clone();

    // we add the prefix "Py" to the receiver
    let py_receiver = add_prefix_to_first_token("Py", receiver.clone());

    // we parse the method signature
    let method: Signature = syn::parse2(method).unwrap();

    let (
        fn_name,
        fn_args_names,
        fn_args_types,
        fn_args_mut,
        fn_args_refs,
        return_type,
        is_void,
        is_static,
    ) = parse_sigature(method);

    let is_option = return_type.to_string().starts_with("Option");
    let is_constructor = fn_name.to_string() == "new";

    let py_return_type = if NEVER_WRAP.contains(&return_type.to_string().as_str()) {
        // if the return type is one of the types that should never be wrapped, we just return it as is
        return_type.clone()
    } else if return_type.to_string() == "Self" {
        // if return type is "Self", we need to replace it with the struct name (PyName)
        add_prefix_to_first_token("Py", receiver.clone())
    } else if return_type.to_string().starts_with("Option") {
        // if it's an Option<> we need to treat it differently
        let inner_type = return_type
            .to_string()
            .replace("Option <", "")
            .replace(">", "");
        let inner_type =
            add_prefix_to_first_token("Py", syn::parse_str(&inner_type).unwrap());
        quote!(#inner_type)
    } else {
        // otherwise we just prepend "Py" to the return type
        add_prefix_to_first_token("Py", return_type)
    };

    // iterate over the arguments and generate the final argument list, taking care of references and mutability
    let mut args_list = Vec::new();

    for (name, ty, is_mut, is_ref) in izip!(
        fn_args_names.iter(),
        fn_args_types.iter(),
        fn_args_mut.iter(),
        fn_args_refs.iter()
    ) {
        let py_ty = if NEVER_WRAP.contains(&ty.to_string().as_str()) {
            ty.clone()
        } else {
            format_ident!("Py{}", ty) as syn::Ident
        };

        if *is_mut && *is_ref {
            args_list.push(quote::quote! {#name: &mut #py_ty});
        } else if *is_ref {
            args_list.push(quote::quote! {#name: &#py_ty});
        } else {
            args_list.push(quote::quote! {#name: #py_ty});
        }
    }

    // let's do the same for calling the method on the inner struct
    let mut inner_args_list = Vec::new();

    for (name, ty, is_mut, is_ref) in izip!(
        fn_args_names.iter(),
        fn_args_types.iter(),
        fn_args_mut.iter(),
        fn_args_refs.iter()
    ) {
        if NEVER_WRAP.contains(&ty.to_string().as_str()) {
            inner_args_list.push(quote::quote! {#name});
        } else if *is_mut && *is_ref {
            inner_args_list.push(quote::quote! {&mut #name.0});
        } else {
            inner_args_list.push(quote::quote! {#name.0});
        }
    }

    let decorator = if is_constructor {
        // if it's a constructor, we need to annotate it with #[new]
        quote::quote! { #[new] }
    } else if is_static {
        // if it's a static method, we need to annotate it with #[staticmethod]
        quote::quote! { #[staticmethod] }
    } else {
        // otherwise we don't need any decorator
        quote::quote! {}
    };

    let method_signature = if is_static {
        quote::quote! {
            #[pymethods]
            impl #py_receiver {
                #decorator
                fn #fn_name(#(#args_list),*) -> #py_return_type {
                    #py_return_type(#receiver::#fn_name(#(#inner_args_list),*))
                }
            }
        }
    } else if is_void {
        quote::quote! {
            #[pymethods]
            impl #py_receiver {
                fn #fn_name(&mut self, #(#args_list),*) -> () {
                    self.0.#fn_name(#(#inner_args_list),*);
                }
            }
        }
    } else if is_option {
        quote::quote! {
            #[pymethods]
            impl #py_receiver {
                fn #fn_name(&mut self, #(#args_list),*) -> Option< #py_return_type > {
                    match self.0.#fn_name(#(#inner_args_list),*) {
                        Some(val) => Some(#py_return_type(val.clone())),
                        None => None,
                    }
                }
            }
        }
    } else {
        quote::quote! {
            #[pymethods]
            impl #py_receiver {
                fn #fn_name(&mut self, #(#args_list),*) -> #py_return_type {
                    self.0.#fn_name(#(#inner_args_list),*).into()
                }
            }
        }
    };

    proc_macro::TokenStream::from(method_signature)
}

fn split_token_stream_by_comma(
    input: proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();
    let mut current = proc_macro2::TokenStream::new();
    for token in input {
        match token {
            proc_macro2::TokenTree::Punct(punct) => {
                if punct.as_char() == ',' {
                    result.push(current);
                    current = proc_macro2::TokenStream::new();
                } else {
                    current.extend(std::iter::once(proc_macro2::TokenTree::Punct(punct)));
                }
            }
            _ => current.extend(std::iter::once(token)),
        }
    }
    result.push(current);
    result
}

/// This macro exposes a read-only field of a wrapped struct to Python.
/// It generates a getter for the field that will be annotated with #[getter].
/// The field must be public.
///
/// Example:
/// ```rust
/// py_getter!(SomeStruct, field, SomeType);
/// ```
///
/// This will generate the following code:  
/// ```rust
/// #[pymethods]
/// impl PySomeStruct {
///    #[getter]
///   pub fn field(&self) -> PySomeType {
///      PySomeType(self.0.field)
///  }
/// }
/// ```
#[proc_macro]
pub fn py_getter(input: TokenStream) -> TokenStream {
    // convert the input to a token stream
    let input = proc_macro2::TokenStream::from(input);

    let splitted = split_token_stream_by_comma(input);
    let receiver = splitted
        .get(0)
        .expect("Expected receiver as first argument")
        .clone();
    let field = splitted
        .get(1)
        .expect("Expected field as second argument")
        .clone();
    let field_type = splitted
        .get(2)
        .expect("Expected field type as third argument")
        .clone();

    let py_receiver = add_prefix_to_first_token("Py", receiver.clone());

    // we generate the getter
    let getter = quote!(
        #[pymethods]
        impl #py_receiver {
            #[getter(#field)]
            pub fn #field(&self) -> #field_type {
                #field_type(self.0.#field)
            }
        }
    );

    proc_macro::TokenStream::from(getter)

    // create the
}

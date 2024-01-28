use quote::quote;

#[proc_macro_attribute]
pub fn attrib_macro_logger_1(args: TokenStream, item: TokenStream) -> TokenStream {
    logger::attrib_proc_macro_impl(args, item)
}

/// The args take a key value pair like `#[attrib_macro_logger(key = "value")]`.
pub fn attrib_proc_macro_impl_1(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    quote! {}.into()
}

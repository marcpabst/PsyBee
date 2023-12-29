extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, DataStruct, DeriveInput, Fields, Type};

#[proc_macro_derive(ToWGSL)]
pub fn to_wgsl_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Couldn't parse item");
    impl_to_wgsl(&ast)
}

fn impl_to_wgsl(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("ToWGSL can only be used with structs with named fields"),
    };

    let field_strings = fields.iter().map(|f| {
        let field_name = &f.ident;
        let wgsl_type = match &f.ty {
            Type::Path(type_path) if type_path.path.is_ident("f32") => "f32",
            Type::Path(type_path) if type_path.path.is_ident("i32") => "i32",
            Type::Path(type_path)
                if type_path
                    .path
                    .segments
                    .iter()
                    .any(|segment| segment.ident == "Vec2") =>
            {
                "vec2<f32>"
            }
            Type::Path(type_path)
                if type_path
                    .path
                    .segments
                    .iter()
                    .any(|segment| segment.ident == "Vec3") =>
            {
                "vec3<f32>"
            }
            _ => panic!("Field type not supported for WGSL"),
        };
        quote! { #field_name: #wgsl_type; }
    });

    let gen = quote! {
        impl ToWGSL for #name {
            fn to_wgsl(&self) -> String {
                format!(concat!("struct ", stringify!(#name), " {\n", #(stringify!(#field_strings), "\n")*, "};\n"))
            }
        }
    };

    gen.into()
}

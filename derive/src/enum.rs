pub(crate) fn impl_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let attribute = ast.attrs.iter().find(|a| {
        a.path.segments.len() == 1 && a.path.segments[0].ident == "r#enum"
    });

    let parameters = match attribute {
        Some(attribute) => {
            syn::parse2(attribute.tokens.clone())
                .expect("Invalid entity attribute!")
        },
        None => crate::Params::default(),
    };

    let variants = match ast.data {
        syn::Data::Enum(ref e) => &e.variants,
        _ => unimplemented!(),
    };

    let name = &ast.ident;
    let elephantry = if parameters.internal {
        quote::quote! {
            crate
        }
    }
    else {
        quote::quote! {
            elephantry
        }
    };

    let from_text_body = variants.iter().map(|variant| {
        let name = &variant.ident;

        quote::quote! {
            stringify!(#name) => Self::#name
        }
    });

    let gen = quote::quote! {
        impl #elephantry::Enum for #name {
            fn name() -> &'static str {
                stringify!(#name)
            }

            fn from_text(value: &str) -> #elephantry::Result<Box<Self>> {
                let v = match value {
                    #(#from_text_body, )*
                    _ => unreachable!(),
                };

                Ok(Box::new(v))
            }
        }
    };

    gen.into()
}
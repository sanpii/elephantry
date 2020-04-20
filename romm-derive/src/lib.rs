extern crate proc_macro;

#[proc_macro_derive(Entity)]
pub fn entity_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
    let ast = syn::parse(input)
        .unwrap();

    impl_entity_macro(&ast)
}

fn impl_entity_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream
{
    let fields = match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => unimplemented!(),
    };

    let from_body = fields.iter()
        .map(|field| {
            let name = &field.ident;

            quote::quote! {
                #name: tuple.get(stringify!(#name))
            }
        });

    let get_body = fields.iter()
        .map(|field| {
            let name = &field.ident;
            let ty = &field.ty;

            if is_option(ty) {
                quote::quote! {
                    stringify!(#name) => match self.#name {
                        Some(ref value) => Some(value),
                        None => None,
                    }
                }
            } else {
                quote::quote! {
                    stringify!(#name) => Some(&self.#name)
                }
            }
        });

    let name = &ast.ident;

    let gen = quote::quote! {
        impl romm::Entity for #name
        {
            fn from(tuple: &romm::pq::Tuple) -> Self
            {
                Self {
                    #(#from_body, )*
                }
            }

            fn get(&self, field: &str) -> Option<&dyn romm::pq::ToSql> {
                match field {
                    #(#get_body, )*
                    _ => None,
                }
            }
        }
    };

    gen.into()
}

fn is_option(ty: &syn::Type) -> bool
{
    let typepath = match ty {
        syn::Type::Path(typepath) => typepath,
        _ => unimplemented!(),
    };

    typepath.path.leading_colon.is_none()
        && typepath.path.segments.len() == 1
        && typepath.path.segments.iter().next().unwrap().ident == "Option"
}

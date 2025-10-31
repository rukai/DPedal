use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, DataEnum, DeriveInput, Expr, FieldsNamed, FieldsUnnamed, Ident, Lit, Meta,
    spanned::Spanned,
};

fn get_finalize_type(attrs: &[Attribute]) -> Result<String, syn::Error> {
    for attr in attrs {
        if let Meta::NameValue(name_value) = &attr.meta {
            if name_value.path.is_ident("kdl_config_finalize_into") {
                if let Expr::Lit(lit) = &name_value.value {
                    if let Lit::Str(lit) = &lit.lit {
                        return Ok(lit.value());
                    }
                }
            }
        }
    }

    Err(syn::Error::new(
        // TODO: combine all attr spans
        attrs.first().span(),
        "`KdlConfigFinalize` missing attribute `#[kdl_config_finalize_into = \"full::path::to::FinalizeType\"`",
    ))
}

pub fn generate(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let ident = input.ident;

    let finalize_type = get_finalize_type(&input.attrs)?;
    let finalize_type: Expr = syn::parse_str(&finalize_type)?;

    match input.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let rust_field_names: Vec<&syn::Ident> =
                    named.iter().map(|x| x.ident.as_ref().unwrap()).collect();
                Ok(quote! {
                    impl KdlConfigFinalize for #ident {
                        type FinalizeType = #finalize_type;
                        fn finalize(&self) -> Self::FinalizeType {
                            Self::FinalizeType {
                                #(
                                    #rust_field_names: self.#rust_field_names.value.finalize(),
                                )*
                            }
                        }
                    }
                })
            }
            syn::Fields::Unnamed(FieldsUnnamed { .. }) => Err(syn::Error::new(
                s.struct_token.span,
                "`KdlConfigFinalize` cannot be derived for unnamed structs",
            )),
            syn::Fields::Unit => Err(syn::Error::new(
                s.struct_token.span,
                "`KdlConfigFinalize` cannot be derived for unit structs",
            )),
        },
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let variant_idents: Vec<&Ident> = variants.iter().map(|v| &v.ident).collect();
            Ok(quote! {
                impl KdlConfigFinalize for #ident {
                    type FinalizeType = #finalize_type;
                    fn finalize(&self) -> Self::FinalizeType {
                        match self {
                            #( #ident::#variant_idents => #finalize_type::#variant_idents, )*
                        }
                    }
                }
            })
        }
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span,
            "`KdlConfigFinalize` cannot be derived for unions",
        )),
    }
}

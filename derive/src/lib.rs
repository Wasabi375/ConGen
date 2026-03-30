use quote::{format_ident, quote};
use syn::{
    Field, GenericArgument, Ident, ItemStruct, Meta, PathArguments, Token, Type,
    parse::Parse, parse_macro_input, parse2, punctuated::Punctuated,
};
#[cfg(feature = "std")]
use syn::{Path, parse_quote};

#[derive(Default)]
struct CongenAttribute {
    default: Option<CongenDefault>,
}

enum CongenDefault {
    UseDefault,
}

impl Parse for CongenAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;

        let mut default = None;

        for arg in args {
            match arg.to_string().as_str() {
                "default" => {
                    if default.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "\"default\" should only be specified once in `congen` attribute",
                        ));
                    }
                    default = Some(CongenDefault::UseDefault);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        arg,
                        "unknown argument to `congen` attribute",
                    ));
                }
            }
        }

        Ok(CongenAttribute { default })
    }
}

struct CongenField {
    attr: CongenAttribute,
    field: Field,
    option_type: Option<Type>,
}

impl CongenField {
    fn from_field(errors: &mut Vec<syn::Error>, field: Field) -> CongenField {
        let attr = field
            .attrs
            .iter()
            .find_map(|attr| match &attr.meta {
                Meta::List(meta_list) if meta_list.path.is_ident(&format_ident!("congen")) => {
                    Some(parse2(meta_list.tokens.clone()))
                }
                _ => None,
            })
            .unwrap_or(Ok(CongenAttribute::default()))
            .unwrap_or_else(|err| {
                errors.push(err);
                CongenAttribute::default()
            });

        let option_type = is_option_type(&field.ty).unwrap_or_else(|err| {
            errors.push(err);
            None
        });

        CongenField {
            attr,
            field,
            option_type,
        }
    }
}

fn is_option_type(ty: &Type) -> Result<Option<Type>, syn::Error> {
    let Type::Path(path) = ty else {
        return Ok(None);
    };

    if path.qself.is_some() {
        return Err(syn::Error::new_spanned(
            ty,
            "Qself path type not supported in option for congen",
        ));
    }

    let path = &path.path;
    if path.leading_colon.is_some() {
        return Ok(None);
    }

    let mut segments = path.segments.iter();
    let Some(first) = segments.next() else {
        return Err(syn::Error::new_spanned(
            ty,
            "Empty path. Is this even possible?",
        ));
    };
    let option_segment = match first.ident.to_string().as_str() {
        "Option" => first.clone(),
        "std" | "core" => {
            let Some(opt) = segments.next() else {
                return Ok(None);
            };
            if opt.ident.to_string() != "option" {
                return Ok(None);
            };
            let Some(opt) = segments.next() else {
                return Ok(None);
            };
            if opt.ident.to_string() != "Option" {
                return Ok(None);
            };
            opt.clone()
        }
        _ => return Ok(None),
    };

    let PathArguments::AngleBracketed(opt_inner_type_args) = option_segment.arguments else {
        return Ok(None);
    };

    if opt_inner_type_args.args.len() != 1 {
        return Ok(None);
    }

    let GenericArgument::Type(inner_ty) = opt_inner_type_args
        .args
        .first()
        .expect("just checked the length")
    else {
        return Ok(None);
    };

    Ok(Some(inner_ty.clone()))
}

#[proc_macro_derive(Configuration, attributes(congen))]
pub fn configuration(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    #[cfg(feature = "std")]
    let vec: Path = parse_quote!(std::vec::Vec);
    #[cfg(not(feature = "std"))]
    let vec: Path = parse_quote!(alloc::vec::Vec);

    let input = parse_macro_input!(input as ItemStruct);

    if matches!(input.fields, syn::Fields::Unnamed(_)) {
        return quote! { compile_error!("Configuration not yet supported for tupple structs") }
            .into();
    }

    let mut errors = Vec::new();
    let fields: Vec<_> = input
        .fields
        .into_iter()
        .map(|field| CongenField::from_field(&mut errors, field))
        .collect();

    let ty = &input.ident;
    let type_name = input.ident.to_string();
    let vis = &input.vis;
    let change_type = format_ident!("{}Change", input.ident);
    let change_fields = fields.iter().map(|field| {
        let ident = &field.field.ident;
        if let Some(ty) = field.option_type.as_ref() {
            quote! {
                #ident: Option<<#ty as congen::Configuration>::CongenChange>
            }
        } else {
            let ty = &field.field.ty;
            quote! { #ident: <#ty as congen::Configuration>::CongenChange }
        }
    });
    let apply_change = fields.iter().map(|field| {
        let ident = &field.field.ident;
        if let Some(ty) = field.option_type.as_ref() {
            quote! {
                todo!("apply change to option of type {}", stringify!(#ty));
            }
        } else {
            let ty = &field.field.ty;
            quote! {
                <#ty as congen::Configuration>::apply_change(&mut self.#ident, change.#ident);
            }
        }
    });
    let field_desc = fields.iter().map(|field| {
        let field_name = if let Some(ident) = &field.field.ident {
            let name = ident.to_string();
            quote! { Some(#name) }
        } else {
            quote! { None }
        };
        let ty = field.option_type.as_ref().unwrap_or(&field.field.ty);

        let as_option = if field.option_type.is_some() {
            quote! { .as_option() }
        } else {
            quote! {}
        };
        let with_default = if field.attr.default.is_some() {
            quote! { .with_default() }
        } else {
            quote! {}
        };

        quote! {
            children.push(<#ty as congen::Configuration>::description(#field_name) #as_option #with_default);
        }
    });
    let field_defaults = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;

        if field.attr.default.is_none() {
            return quote! {
                #ident: { return Err(congen::NotSupported) }
            };
        }

        if field.option_type.is_some() {
            quote! {
                #ident: None
            }
        } else {
            quote! {
                #ident: <#ty as congen::Configuration>::default()?
            }
        }
    });

    let errors = errors.iter().map(|e| e.to_compile_error());

    quote! {

        #(#errors)*

        #[doc = concat!("Change type for [`", stringify!(#ty), "`] in use with [`congen::Configuration`]")]
        #vis struct #change_type {
            #(#change_fields),*
        }

        impl congen::Configuration for #ty {
            type CongenChange  = #change_type;

            fn apply_change(&mut self, change: #change_type) {
                #(#apply_change)*
            }

            fn description(field_name: Option<&'static str>) -> congen::Description {
                let mut children = #vec::new();

                #(#field_desc)*

                congen::Description::Composit(
                    congen::CompositDescription {
                        field_name,
                        type_name: Self::type_name(),
                        children,
                        has_default: false, // TODO how to fill this?
                        allow_unset: false,
                    }
                )
            }

            fn default() -> Result<Self, congen::NotSupported> {
                Ok(Self {
                    #(#field_defaults),*
                })
            }

            fn type_name() -> std::borrow::Cow<'static, str> {
                #type_name.into()
            }
        }
    }
    .into()
}

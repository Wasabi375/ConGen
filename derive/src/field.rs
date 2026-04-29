use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, Field, GenericArgument, Ident, Meta, PathArguments, Token, Type, parse::Parse, parse2,
    punctuated::Punctuated,
};

pub enum AttributeParam {
    Flag(Ident),
    NameValue {
        ident: Ident,
        value: Expr,
        eq: Token![=],
    },
}

impl AttributeParam {
    fn ident(&self) -> &Ident {
        match self {
            AttributeParam::Flag(ident) => ident,
            AttributeParam::NameValue { ident, .. } => ident,
        }
    }
}

impl Parse for AttributeParam {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;

        if input.peek(Token![=]) {
            let eq = input.parse()?;
            let value = input.parse()?;

            Ok(AttributeParam::NameValue { ident, value, eq })
        } else {
            Ok(AttributeParam::Flag(ident))
        }
    }
}

impl ToTokens for AttributeParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            AttributeParam::Flag(ident) => ident.to_tokens(tokens),
            AttributeParam::NameValue { ident, value, eq } => {
                ident.to_tokens(tokens);
                eq.to_tokens(tokens);
                value.to_tokens(tokens);
            }
        }
    }
}

#[derive(Default)]
pub struct CongenAttribute {
    pub default: Option<CongenDefault>,
    pub inner_default: Option<Expr>,
}

pub enum CongenDefault {
    UseDefault,
    Expr(Expr),
}

impl Parse for CongenAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<AttributeParam, Token![,]>::parse_terminated(input)?;

        let mut default = None;
        let mut inner_default = None;

        for arg in args {
            match arg.ident().to_string().as_str() {
                "default" => {
                    if default.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "\"default\" should only be specified once in `congen` attribute",
                        ));
                    }

                    default = Some(match arg {
                        AttributeParam::Flag(_ident) => CongenDefault::UseDefault,
                        AttributeParam::NameValue { value, .. } => CongenDefault::Expr(value),
                    });
                }
                "inner-default" => {
                    if inner_default.is_some() {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "\"inner_default\" should only be specified once in `congen` attribute",
                        ));
                    }

                    let AttributeParam::NameValue { value: expr, .. } = arg else {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "\"inner_default\" requires expression as argument",
                        ));
                    };

                    inner_default = Some(expr);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        arg,
                        "unknown argument to `congen` attribute",
                    ));
                }
            }
        }

        Ok(CongenAttribute {
            default,
            inner_default,
        })
    }
}

pub struct CongenField {
    pub attr: CongenAttribute,
    pub field: Field,
    pub option_type: Option<Type>,
}

impl CongenField {
    pub fn from_field(errors: &mut Vec<syn::Error>, field: Field) -> CongenField {
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

    pub fn derive_default_constructor(&self) -> TokenStream {
        if let Some(inner_default) = self.attr.inner_default.as_ref() {
            if self.option_type.is_none() {
                return syn::Error::new_spanned(
                    &self.field,
                    "\"inner_default\" parameter is only allowed on optional fields",
                )
                .into_compile_error();
            }
            quote! {
                Some(|| { Some(#inner_default) })
            }
        } else if let Some(default) = self.attr.default.as_ref() {
            match default {
                CongenDefault::Expr(expr) => quote! { Some(|| { #expr }) },
                CongenDefault::UseDefault => {
                    let field_ty = &self.field.ty;

                    if let Some(option_type) = self.option_type.as_ref() {
                        quote! {
                            Some(|| {
                                <#option_type as congen::Configuration>::default().map(|d| Some(d))
                                    .or_else(|_| { <#field_ty as congen::Configuration>::default() })
                                    .expect(&format!("field is marked as \"use_default\", but `default` is not implemented for {}", core::any::type_name::<#field_ty>()))
                            })
                        }
                    } else {
                        quote! {
                            Some(|| {
                                <#field_ty as congen::Configuration>::default()
                                    .expect(&format!("field is marked as \"use_default\", but `default` is not implemented for {}", core::any::type_name::<#field_ty>()))
                            })
                        }
                    }
                }
            }
        } else {
            if let Some(option_type) = self.option_type.as_ref() {
                let field_ty = &self.field.ty;
                quote! {
                    Some(|| {
                        <#option_type as congen::Configuration>::default().map(|d| Some(d))
                            .or_else(|_| { <#field_ty as congen::Configuration>::default() })
                            .expect(&format!("field is optional, but `default` is not implemented for {}", core::any::type_name::<#field_ty>()))
                    })
                }
            } else {
                quote! {
                    None
                }
            }
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
            if opt.ident != "option" {
                return Ok(None);
            };
            let Some(opt) = segments.next() else {
                return Ok(None);
            };
            if opt.ident != "Option" {
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

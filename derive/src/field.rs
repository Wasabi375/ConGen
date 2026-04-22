use quote::format_ident;
use syn::{
    Field, GenericArgument, Ident, Meta, PathArguments, Token, Type, parse::Parse, parse2,
    punctuated::Punctuated,
};

#[derive(Default)]
pub struct CongenAttribute {
    pub default: Option<CongenDefault>,
}

pub enum CongenDefault {
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

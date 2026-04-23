use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemStruct, parse_macro_input};
use syn::{Path, parse_quote};

use crate::field::CongenField;

mod field;

#[proc_macro_derive(Configuration, attributes(congen))]
pub fn configuration(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let vec: Path = parse_quote!(std::vec::Vec);

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
    let change_fields_decls = fields.iter().map(|field| {
        let ident = &field.field.ident;
        if let Some(ty) = field.option_type.as_ref() {
            quote! {
                #ident: congen::OptionChange<<#ty as congen::Configuration>::CongenChange>
            }
        } else {
            let ty = &field.field.ty;
            quote! { #ident: <#ty as congen::Configuration>::CongenChange }
        }
    });

    let apply_change = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;
        quote! {
            <#ty as congen::Configuration>::apply_change(&mut self.#ident, change.#ident);
        }
    });
    let field_desc = fields.iter().map(|field| {
        let field_name = if let Some(ident) = &field.field.ident {
            let name = ident.to_string();
            quote! { #name }
        } else {
            quote! { compile_error!("Configuration not supported for tupple structs") }
        };
        let ty = &field.field.ty;

        let with_default = if field.attr.default.is_some() {
            quote! { .with_default() }
        } else {
            quote! {}
        };

        quote! {
            children.push(<#ty as congen::Configuration>::description(#field_name) #with_default);
        }
    });
    let mut has_default = true;
    let field_defaults = fields.iter().map(|field| {
        let ident = &field.field.ident;
        let ty = &field.field.ty;

        let Some(default) = field.attr.default.as_ref() else {
            has_default = false;
            return quote! {
                #ident: { return Err(congen::NotSupported) }
            };
        };

        match default {
            field::CongenDefault::UseDefault => quote! {
                #ident: <#ty as congen::Configuration>::default()?
            },
            field::CongenDefault::Expr(expr) => quote! {
                #ident: #expr
            },
        }
    });

    let congen_change_impl = derive_congen_change(&change_type, &fields);

    let errors = errors.iter().map(|e| e.to_compile_error());

    quote! {

        #(#errors)*

        #[doc = concat!("Change type for [`", stringify!(#ty), "`] in use with [`congen::Configuration`]")]
        #[derive(Default)]
        #vis struct #change_type {
            #(#change_fields_decls),*
        }

        impl congen::Configuration for #ty {
            type CongenChange  = #change_type;

            fn apply_change(&mut self, change: #change_type) {
                #(#apply_change)*
            }

            fn description(field_name: &'static str) -> congen::Description {
                let mut children = #vec::new();

                #(#field_desc)*

                congen::Description::Composit(
                    congen::CompositDescription {
                        field_name,
                        type_name: Self::type_name(),
                        fields: children,
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

        #congen_change_impl
    }
    .into()
}

fn derive_congen_change(change_type: &Ident, fields: &[CongenField]) -> TokenStream {
    let field_idents: Vec<_> = fields.iter().map(|field| &field.field.ident).collect();

    let fields_from_path = fields.iter().map(|field| {
        let ident = &field.field.ident.clone().expect("tuples are not supported");
        let ident_str = ident.to_string();
        let field_ty = field.field.ty.clone();

        let special_case_default_verb = match &field.attr.default {
            Some(field::CongenDefault::Expr(default_expr)) => {
                quote! {
                    let mut path = path.peekable();
                    let verb = if path.peek().is_none() && matches!(verb, congen::ChangeVerb::UseDefault) {
                        let value: #field_ty = #default_expr;
                        congen::ChangeVerb::SetAny(Box::new(value))
                    } else {
                        verb
                    };
                }
            },
            _ => quote! {}
        };

        quote! {
            Some(#ident_str) => {
                #special_case_default_verb
                congen::CongenChange::apply_change(
                    &mut change.#ident,
                    <#field_ty as Configuration>::CongenChange::from_path_and_verb(path, verb)?,
                );
            }
        }
    });
    quote! {
        impl congen::CongenChange for #change_type {
            fn empty() -> Self {
                #change_type {
                    #(#field_idents: congen::CongenChange::empty()),*
                }
            }

            fn apply_change(&mut self, change: Self) {
                #(congen::CongenChange::apply_change(&mut self.#field_idents, change.#field_idents));*
            }

            fn from_path_and_verb<'a, P>(
                mut path: P,
                verb: congen::ChangeVerb)
            -> Result<Self, congen::VerbError>
            where P: Iterator<Item = &'a str> {
                let field_name = path.next();
                let mut change = Self::empty();
                match field_name {
                    #(#fields_from_path,)*
                    Some(_) => return Err(congen::VerbError::InvalidPath),
                    None => todo!("support unset and use-default verbs"),
                };
                Ok(change)
            }
        }
    }
}

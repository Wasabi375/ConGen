use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, ItemStruct, Visibility, parse_macro_input};

use crate::field::{CongenDefault, CongenField};

mod field;

#[proc_macro_derive(ValueEnumConfiguration)]
pub fn value_enum_configuration(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ty = &input.ident;

    if !matches!(input.data, syn::Data::Enum(_)) {
        return quote! { compile_error!("ValueEnumConfiguration can only be derived for enums") }
            .into();
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics congen::ValueEnumConfiguration for #ty #ty_generics #where_clause {}
    }
    .into()
}

#[proc_macro_derive(Configuration, attributes(congen))]
pub fn configuration(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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

    let vis = &input.vis;
    let change_type = format_ident!("{}Change", input.ident);

    let change_type_impl = derive_change_type(&input.ident, &change_type, vis, &fields);
    let configuration_impl = derive_configuration_impl(&input.ident, &change_type, &fields);
    let congen_change_impl = derive_congen_change(&input.ident, &change_type, &fields);

    let errors = errors.iter().map(|e| e.to_compile_error());

    quote! {

        #(#errors)*

        #change_type_impl

        #configuration_impl

        #congen_change_impl
    }
    .into()
}

#[expect(unused)] // TODO
fn derive_option_impl(ty: &Ident, change_type: &Ident, fields: &[CongenField]) -> TokenStream {
    let has_default = fields.iter().fold(true, |acc, field| {
        let field_has_default = field.attr.default.is_some();
        acc && field_has_default
    });
    if !has_default {
        return quote! {};
    }

    #[expect(unused)]
    let fields_requiring_default = fields
        .iter()
        .filter(|field| matches!(field.attr.default, Some(CongenDefault::Expr(_))));

    // TODO I want to somehow test at comp-time that field has a default implementation in the
    // Configuration trait impl
    quote! {}
}

fn derive_configuration_impl(
    ty: &Ident,
    change_type: &Ident,
    fields: &[CongenField],
) -> TokenStream {
    let type_name = ty.to_string();

    let apply_change = fields.iter().map(|field| {
        let ident = field.field.ident.as_ref().expect("tuple structs are not supported");
        let ty = &field.field.ty;
        let default_constructor = field.derive_default_constructor();
        quote! {
            <#ty as congen::Configuration>::apply_change_with_default(&mut self.#ident, change.#ident, #default_constructor);
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

    let field_defaults = fields
        .iter()
        .map(|field| {
            let ident = &field
                .field
                .ident
                .as_ref()
                .expect("Tuple structs not supported");
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
        })
        .collect::<Vec<_>>();

    quote! {
        impl congen::Configuration for #ty {
            type CongenChange  = #change_type;

            fn apply_change_with_default(&mut self, change: #change_type, default: Option<fn() -> Self>) {
                #(#apply_change)*
            }

            fn description(field_name: &'static str) -> congen::Description {
                let mut children = std::vec::Vec::new(); // TODO based on a feature this should use
                                                         // alloc::vec::Vec instead

                #(#field_desc)*

                congen::Description::Composit(
                    congen::CompositDescription {
                        field_name,
                        type_name: Self::type_name(),
                        fields: children,
                        has_default: #has_default,
                        allow_unset: false,
                    }
                )
            }

            fn default() -> Result<Self, congen::NotSupported> {
                #[allow(unreachable_code)]
                Ok(Self {
                    #(#field_defaults),*
                })
            }

            fn type_name() -> std::borrow::Cow<'static, str> {
                #type_name.into()
            }
        }
    }
}

fn derive_change_type(
    config_type: &Ident,
    change_type: &Ident,
    vis: &Visibility,
    fields: &[CongenField],
) -> TokenStream {
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

    quote! {
        #[doc = concat!("Change type for [`", stringify!(#config_type), "`] in use with [`congen::Configuration`]")]
        #[derive(Default, Debug)]
        #vis struct #change_type {
            #(#change_fields_decls),*
        }
    }
}

fn derive_congen_change(
    config_type: &Ident,
    change_type: &Ident,
    fields: &[CongenField],
) -> TokenStream {
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

    let field_path_names = fields.iter().map(|field| {
        field
            .field
            .ident
            .as_ref()
            .expect("Tuple structs not supported")
            .to_string()
    });

    quote! {
        impl congen::CongenChange for #change_type {
            type Configuration = #config_type;

            fn empty() -> Self {
                #change_type {
                    #(#field_idents: congen::CongenChange::empty()),*
                }
            }

            fn default() -> Result<Self, congen::NotSupported> {
                fn map_err(err: congen::VerbError) -> congen::NotSupported {
                    match err {
                        congen::VerbError::InvalidPath | congen::VerbError::ParseError(_) | congen::VerbError::DowncastFailed =>
                            panic!("unexpected error while creating default CongenChange: {err}"),
                        congen::VerbError::NotSupported(_) | congen::VerbError::UnsupportedVerb(_) => congen::NotSupported
                    }
                }

                let mut default = Self::empty();

                #(
                    eprintln!("create default for: {}", #field_path_names);

                    default.apply_change(
                        #change_type::from_path_and_verb([#field_path_names].into_iter(), congen::ChangeVerb::UseDefault)
                            .map_err(map_err)?
                    );
                )*
                eprintln!("done!");

                Ok(default)
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
                    None => {
                        return match verb {
                            congen::ChangeVerb::Set(_) | congen::ChangeVerb::SetAny(_)
                                => Err(congen::VerbError::UnsupportedVerb(verb)),
                            congen::ChangeVerb::SetFlag | congen::ChangeVerb::Unset => {
                                eprintln!("set-flag and unset are not supported by derived congens");
                                Err(congen::VerbError::UnsupportedVerb(verb))
                            },
                            congen::ChangeVerb::UseDefault => Ok(<Self as congen::CongenChange>::default()?),
                        }
                    },
                };
                Ok(change)
            }
        }
    }
}
